use {
    super::TextArea,
    crate::widgets::textarea::behaviour::{cursor::CursorMove, scroll::Scrolling},
    ratatui::{layout::Alignment, style::Style, widgets::Block},
};

impl<'a> TextArea<'a> {
    /// Get the current style of textarea.
    pub fn style(&self) -> Style {
        self.style
    }

    /// Remove the block of textarea
    pub fn remove_block(&mut self) {
        self.block = None;
    }

    /// Get the block of textarea if exists.
    pub fn block<'s>(&'s self) -> Option<&'s Block<'a>> {
        self.block.as_ref()
    }

    /// Get the placeholder text. An empty string means the placeholder is disabled. The default
    /// value is an empty string.
    pub fn placeholder_text(&self) -> &'_ str {
        self.placeholder.as_str()
    }

    /// Get the placeholder style. When the placeholder text is empty, it returns `None` since the
    /// placeholder is disabled. The default style is a dark gray text.
    pub fn placeholder_style(&self) -> Option<Style> {
        if self.placeholder.is_empty() {
            None
        } else {
            Some(self.placeholder_style)
        }
    }

    /// Get the style of cursor.
    pub fn cursor_style(&self) -> Style {
        self.cursor_style
    }

    /// Get slice of line texts. This method borrows the content, but not moves. Note that the
    /// returned slice will never be empty because an empty text means a slice containing one empty
    /// line. This is correct since any text file must end with a newline.
    pub fn lines(&'a self) -> &'a [String] {
        &self.lines
    }

    /// Convert [`TextArea`] instance into line texts.
    pub fn into_lines(self) -> Vec<String> {
        self.lines
    }

    /// Get the current cursor position. 0-base character-wise (row, col) cursor position.
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Get the current selection range as a pair of the start position and the end position. The
    /// range is bounded inclusively below and exclusively above. The positions are 0-base
    /// character-wise (row, col) values. The first element of the pair is always smaller than the
    /// second one even when it is ahead of the cursor. When no text is selected, this method
    /// returns `None`.
    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        self.selection_start.map(|pos| {
            if pos > self.cursor {
                (self.cursor, pos)
            } else {
                (pos, self.cursor)
            }
        })
    }

    /// Get current text alignment. The default alignment is [`Alignment::Left`].
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    /// Check if the textarea has a empty content.
    pub fn is_empty(&self) -> bool {
        self.lines == [""]
    }

    /// Get the yanked text. Text is automatically yanked when deleting strings by
    /// [`TextArea::delete_str`] When multiple lines were yanked, they are always with `\n`.
    pub fn yank_text(&self) -> String {
        self.yank.to_string()
    }

    /// Set a yanked text. `\n` and `\r\n` are recognized as newline but `\r` isn't.
    pub fn set_yank_text(&mut self, text: impl Into<String>) {
        // `str::lines` is not available since it strips a newline at end
        let lines: Vec<_> = text
            .into()
            .split('\n')
            .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
            .collect();
        self.yank = lines.into();
    }

    /// Scroll the textarea. See [`Scrolling`] for the argument.
    /// The cursor will not move until it goes out the viewport. When the cursor position is outside
    /// the viewport after scroll, the cursor position will be adjusted to stay in the viewport
    /// using the same logic as [`CursorMove::InViewport`].
    pub fn scroll(&mut self, scrolling: impl Into<Scrolling>) {
        self.scroll_with_shift(scrolling.into(), self.selection_start.is_some());
    }

    fn scroll_with_shift(&mut self, scrolling: Scrolling, shift: bool) {
        if shift && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }
        scrolling.scroll(&mut self.viewport);
        self.move_cursor_with_shift(CursorMove::InViewport, shift);
    }
}
