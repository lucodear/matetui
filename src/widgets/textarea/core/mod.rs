pub mod builder;
pub mod getset;
pub mod validation;
pub mod widget;

use {
    super::behaviour::{
        cursor::CursorMove,
        highlight::LineHighlighter,
        input::Input,
        util::{spaces, Pos},
    },
    ratatui::{
        layout::Alignment,
        style::{Color, Modifier, Style},
        text::Line,
        widgets::Block,
    },
    std::{
        cmp::Ordering,
        fmt::{self, Debug},
    },
    unicode_width::UnicodeWidthChar as _,
    validation::ValidatorFn,
    widget::Viewport,
};

#[derive(Debug, Clone)]
enum YankText {
    Piece(String),
    Chunk(Vec<String>),
}

impl Default for YankText {
    fn default() -> Self {
        Self::Piece(String::new())
    }
}

impl From<String> for YankText {
    fn from(s: String) -> Self {
        Self::Piece(s)
    }
}
impl From<Vec<String>> for YankText {
    fn from(mut c: Vec<String>) -> Self {
        match c.len() {
            0 => Self::default(),
            1 => Self::Piece(c.remove(0)),
            _ => Self::Chunk(c),
        }
    }
}

impl fmt::Display for YankText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Piece(s) => write!(f, "{}", s),
            Self::Chunk(ss) => write!(f, "{}", ss.join("\n")),
        }
    }
}

/// A type to manage state of textarea. These are some important methods:
#[derive(Clone, Debug)]
pub struct TextArea<'a> {
    pub(crate) viewport: Viewport,
    pub(crate) cursor_style: Style,
    pub(crate) placeholder: String,
    pub(crate) placeholder_style: Style,
    lines: Vec<String>,
    block: Option<Block<'a>>,
    style: Style,
    cursor: (usize, usize), // 0-base
    tab_len: u8,
    cursor_line_style: Style,
    yank: YankText,
    alignment: Alignment,
    mask: Option<char>,
    selection_start: Option<(usize, usize)>,
    select_style: Style,
    validators: Vec<ValidatorFn>,
}

impl<'a, I> From<I> for TextArea<'a>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    /// Convert any iterator whose elements can be converted into [`String`] into [`TextArea`]. Each
    /// [`String`] element is handled as line. Ensure that the strings don't contain any newlines.
    /// This method is useful to create [`TextArea`] from [`std::str::Lines`].
    ///
    /// ```
    /// use tui::widgets::TextArea;
    /// let textarea = TextArea::from(["hello", "world"]);
    /// ```
    fn from(i: I) -> Self {
        Self::new(i.into_iter().map(|s| s.into()).collect::<Vec<String>>())
    }
}

impl<'a> Default for TextArea<'a> {
    /// Create [`TextArea`] instance with empty text content.
    fn default() -> Self {
        Self::new(vec![String::new()])
    }
}

impl<'a> TextArea<'a> {
    /// Create [`TextArea`] instance with given lines.
    pub fn new(mut lines: Vec<String>) -> Self {
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            lines,
            block: None,
            style: Style::default(),
            cursor: (0, 0),
            tab_len: 2,
            cursor_line_style: Style::default(),
            viewport: Viewport::default(),
            cursor_style: Style::default().add_modifier(Modifier::REVERSED),
            yank: YankText::default(),
            alignment: Alignment::Left,
            placeholder: String::new(),
            placeholder_style: Style::default().fg(Color::DarkGray),
            mask: None,
            selection_start: None,
            select_style: Style::default().bg(Color::LightBlue),
            validators: Vec::new(),
        }
    }

    /// Handle a key input with default key mappings.
    /// Recieves any `impl Into<Input>`, e.g. `crossterm`'s key event types. [`Input`] so this
    /// method can take the event values directly. This method returns if the input modified text
    /// contents or not in the textarea.
    pub fn input(&mut self, input: impl Into<Input>) -> bool {
        let input = input.into();
        let modified = match input.kind() {
            ":char" => {
                if let Some(c) = input.maybe_char() {
                    self.insert_char(c);
                    true
                } else {
                    false
                }
            }
            ":non-enter-newline" => self.insert_newline(),
            ":tab" => self.insert_tab(),
            ":backspace" => self.delete_char(),
            ":delete" => self.delete_next_char(),
            ":down" => self.move_cursor_with_shift(CursorMove::Down, input.shift),
            ":up" => self.move_cursor_with_shift(CursorMove::Up, input.shift),
            ":right" => self.move_cursor_with_shift(CursorMove::Forward, input.shift),
            ":left" => self.move_cursor_with_shift(CursorMove::Back, input.shift),
            ":home" => self.move_cursor_with_shift(CursorMove::Head, input.shift),
            ":end" => self.move_cursor_with_shift(CursorMove::End, input.shift),
            ":word-right" => self.move_cursor_with_shift(CursorMove::WordForward, input.shift),
            ":word-left" => self.move_cursor_with_shift(CursorMove::WordBack, input.shift),
            _ => false,
        };

        // Check invariants
        debug_assert!(!self.lines.is_empty(), "no line after {:?}", input);
        let (r, c) = self.cursor;
        debug_assert!(
            self.lines.len() > r,
            "cursor {:?} exceeds max lines {} after {:?}",
            self.cursor,
            self.lines.len(),
            input,
        );
        debug_assert!(
            self.lines[r].chars().count() >= c,
            "cursor {:?} exceeds max col {} at line {:?} after {:?}",
            self.cursor,
            self.lines[r].chars().count(),
            self.lines[r],
            input,
        );

        modified
    }

    /// Insert a single character at current cursor position.
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' || c == '\r' {
            self.insert_newline();
            return;
        }

        self.delete_selection(false);
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let i = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
        line.insert(i, c);
        self.cursor.1 += 1;
    }

    /// Insert a string at current cursor position. This method returns if some text was inserted or
    /// not in the textarea. Both `\n` and `\r\n` are recognized as newlines but `\r` isn't.
    pub fn insert_str<S: AsRef<str>>(&mut self, s: S) -> bool {
        let modified = self.delete_selection(false);
        let mut lines: Vec<_> =
            s.as_ref().split('\n').map(|s| s.strip_suffix('\r').unwrap_or(s).to_string()).collect();
        match lines.len() {
            0 => modified,
            1 => self.insert_piece(lines.remove(0)),
            _ => self.insert_chunk(lines),
        }
    }

    fn insert_chunk(&mut self, chunk: Vec<String>) -> bool {
        debug_assert!(chunk.len() > 1, "Chunk size must be > 1: {:?}", chunk);

        let (row, _col) = self.cursor;
        let (row, col) = (row + chunk.len() - 1, chunk[chunk.len() - 1].chars().count());
        self.cursor = (row, col);
        true
    }

    fn insert_piece(&mut self, s: String) -> bool {
        if s.is_empty() {
            return false;
        }

        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        debug_assert!(
            !s.contains('\n'),
            "string given to TextArea::insert_piece must not contain newline: {:?}",
            line,
        );

        let i = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
        line.insert_str(i, &s);

        self.cursor.1 += s.chars().count();
        true
    }

    fn delete_range(&mut self, start: Pos, end: Pos, should_yank: bool) {
        self.cursor = (start.row, start.col);

        if start.row == end.row {
            let removed =
                self.lines[start.row].drain(start.offset..end.offset).as_str().to_string();
            if should_yank {
                self.yank = removed.clone().into();
            }
            return;
        }

        let mut deleted = vec![self.lines[start.row].drain(start.offset..).as_str().to_string()];
        deleted.extend(self.lines.drain(start.row + 1..end.row));
        if start.row + 1 < self.lines.len() {
            let mut last_line = self.lines.remove(start.row + 1);
            self.lines[start.row].push_str(&last_line[end.offset..]);
            last_line.truncate(end.offset);
            deleted.push(last_line);
        }

        if should_yank {
            self.yank = YankText::Chunk(deleted.clone());
        }
    }

    /// Delete a string from the current cursor position. The `chars` parameter means number of
    /// characters, not a byte length of the string. Newlines at the end of lines are counted in the
    /// number. This method returns if some text was deleted or not.
    pub fn delete_str(&mut self, chars: usize) -> bool {
        if self.delete_selection(false) {
            return true;
        }
        if chars == 0 {
            return false;
        }

        let (start_row, start_col) = self.cursor;

        let mut remaining = chars;
        let mut find_end = move |line: &str| {
            let mut col = 0usize;
            for (i, _) in line.char_indices() {
                if remaining == 0 {
                    return Some((i, col));
                }
                col += 1;
                remaining -= 1;
            }
            if remaining == 0 {
                Some((line.len(), col))
            } else {
                remaining -= 1;
                None
            }
        };

        let line = &self.lines[start_row];
        let start_offset =
            { line.char_indices().nth(start_col).map(|(i, _)| i).unwrap_or(line.len()) };

        // First line
        if let Some((offset_delta, _col_delta)) = find_end(&line[start_offset..]) {
            let end_offset = start_offset + offset_delta;
            let removed =
                self.lines[start_row].drain(start_offset..end_offset).as_str().to_string();
            self.yank = removed.clone().into();
            return true;
        }

        let mut r = start_row + 1;
        let mut offset = 0;
        let mut col = 0;

        while r < self.lines.len() {
            let line = &self.lines[r];
            if let Some((o, c)) = find_end(line) {
                offset = o;
                col = c;
                break;
            }
            r += 1;
        }

        let start = Pos::new(start_row, start_col, start_offset);
        let end = Pos::new(r, col, offset);
        self.delete_range(start, end, true);
        true
    }

    /// Insert a tab at current cursor position. Note that this method does nothing when the tab
    /// length is 0. This method returns if a tab string was inserted or not in the textarea.
    pub fn insert_tab(&mut self) -> bool {
        let modified = self.delete_selection(false);
        if self.tab_len == 0 {
            return modified;
        }

        let (row, col) = self.cursor;
        let width: usize = self.lines[row].chars().take(col).map(|c| c.width().unwrap_or(0)).sum();
        let len = self.tab_len - (width % self.tab_len as usize) as u8;
        self.insert_piece(spaces(len).to_string())
    }

    /// Insert a newline at current cursor position.
    pub fn insert_newline(&mut self) -> bool {
        self.delete_selection(false);

        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let offset = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
        let next_line = line[offset..].to_string();
        line.truncate(offset);

        self.lines.insert(row + 1, next_line);
        self.cursor = (row + 1, 0);
        true
    }

    /// Delete a newline from **head** of current cursor line. This method returns if a newline was
    /// deleted or not in the textarea. When some text is selected, it is deleted instead.
    pub fn delete_newline(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }

        let (row, _) = self.cursor;
        if row == 0 {
            return false;
        }

        let line = self.lines.remove(row);
        let prev_line = &mut self.lines[row - 1];

        self.cursor = (row - 1, prev_line.chars().count());
        prev_line.push_str(&line);
        true
    }

    /// Delete one character before cursor. When the cursor is at head of line, the newline before
    /// the cursor will be removed. This method returns if some text was deleted or not in the
    /// textarea. When some text is selected, it is deleted instead.
    pub fn delete_char(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }

        let (row, col) = self.cursor;
        if col == 0 {
            return self.delete_newline();
        }

        let line = &mut self.lines[row];
        if let Some((offset, _c)) = line.char_indices().nth(col - 1) {
            line.remove(offset);
            self.cursor.1 -= 1;
            true
        } else {
            false
        }
    }

    /// Delete one character next to cursor. When the cursor is at end of line, the newline next to
    /// the cursor will be removed. This method returns if a character was deleted or not in the
    /// textarea.
    pub fn delete_next_char(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }

        let before = self.cursor;
        self.move_cursor_with_shift(CursorMove::Forward, false);
        if before == self.cursor {
            return false; // Cursor didn't move, meant no character at next of cursor.
        }

        self.delete_char()
    }

    /// Start text selection at the cursor position. If text selection is already ongoing, the start
    /// position is reset.
    pub fn start_selection(&mut self) {
        self.selection_start = Some(self.cursor);
    }

    /// Stop the current text selection. This method does nothing if text selection is not ongoing.
    pub fn cancel_selection(&mut self) {
        self.selection_start = None;
    }

    fn line_offset(&self, row: usize, col: usize) -> usize {
        let line = self.lines.get(row).unwrap_or(&self.lines[self.lines.len() - 1]);
        line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len())
    }

    /// Set the style used for text selection. The default style is light blue.
    pub fn set_selection_style(&mut self, style: Style) {
        self.select_style = style;
    }

    /// Get the style used for text selection.
    pub fn selection_style(&mut self) -> Style {
        self.select_style
    }

    fn selection_positions(&self) -> Option<(Pos, Pos)> {
        let (sr, sc) = self.selection_start?;
        let (er, ec) = self.cursor;
        let (so, eo) = (self.line_offset(sr, sc), self.line_offset(er, ec));
        let s = Pos::new(sr, sc, so);
        let e = Pos::new(er, ec, eo);
        match (sr, so).cmp(&(er, eo)) {
            Ordering::Less => Some((s, e)),
            Ordering::Equal => None,
            Ordering::Greater => Some((e, s)),
        }
    }

    fn take_selection_positions(&mut self) -> Option<(Pos, Pos)> {
        let range = self.selection_positions();
        self.cancel_selection();
        range
    }

    fn delete_selection(&mut self, should_yank: bool) -> bool {
        if let Some((s, e)) = self.take_selection_positions() {
            self.delete_range(s, e, should_yank);
            return true;
        }
        false
    }

    fn move_cursor_with_shift(&mut self, m: CursorMove, shift: bool) -> bool {
        if let Some(cursor) = m.next_cursor(self.cursor, &self.lines, &self.viewport) {
            if shift {
                if self.selection_start.is_none() {
                    self.start_selection();
                }
            } else {
                self.cancel_selection();
            }
            self.cursor = cursor;
        }
        false
    }

    pub(crate) fn line_spans<'b>(&'b self, line: &'b str, row: usize) -> Line<'b> {
        let mut hl = LineHighlighter::new(
            line,
            self.cursor_style,
            self.tab_len,
            self.mask,
            self.select_style,
        );

        if row == self.cursor.0 {
            hl.cursor_line(self.cursor.1, self.cursor_line_style);
        }

        if let Some((start, end)) = self.selection_positions() {
            hl.selection(row, start.row, start.offset, end.row, end.offset);
        }

        hl.into_spans()
    }
}
