use {
    super::util::spaces,
    ratatui::{
        style::Style,
        text::{Line, Span},
    },
    std::{borrow::Cow, cmp::Ordering, iter},
    unicode_width::UnicodeWidthChar as _,
};

enum Boundary {
    Cursor(Style),
    Select(Style),
    End,
}

impl Boundary {
    fn cmp(&self, other: &Boundary) -> Ordering {
        fn rank(b: &Boundary) -> u8 {
            match b {
                Boundary::Cursor(_) => 3,
                Boundary::Select(_) => 1,
                Boundary::End => 0,
            }
        }
        rank(self).cmp(&rank(other))
    }

    fn style(&self) -> Option<Style> {
        match self {
            Boundary::Cursor(s) => Some(*s),
            Boundary::Select(s) => Some(*s),
            Boundary::End => None,
        }
    }
}

struct DisplayTextBuilder {
    tab_len: u8,
    width: usize,
    mask: Option<char>,
}

impl DisplayTextBuilder {
    fn new(tab_len: u8, mask: Option<char>) -> Self {
        Self {
            tab_len,
            width: 0,
            mask,
        }
    }

    fn build<'s>(&mut self, s: &'s str) -> Cow<'s, str> {
        if let Some(ch) = self.mask {
            // Note: We don't need to track width on masking text since width of tab character is fixed
            let masked = iter::repeat(ch).take(s.chars().count()).collect();
            return Cow::Owned(masked);
        }

        let tab = spaces(self.tab_len);
        let mut buf = String::new();
        for (i, c) in s.char_indices() {
            if c == '\t' {
                if buf.is_empty() {
                    buf.reserve(s.len());
                    buf.push_str(&s[..i]);
                }
                if self.tab_len > 0 {
                    let len = self.tab_len as usize - (self.width % self.tab_len as usize);
                    buf.push_str(&tab[..len]);
                    self.width += len;
                }
            } else {
                if !buf.is_empty() {
                    buf.push(c);
                }
                self.width += c.width().unwrap_or(0);
            }
        }

        if !buf.is_empty() {
            Cow::Owned(buf)
        } else {
            Cow::Borrowed(s)
        }
    }
}

pub struct LineHighlighter<'a> {
    line: &'a str,
    spans: Vec<Span<'a>>,
    boundaries: Vec<(Boundary, usize)>, // TODO: Consider smallvec
    style_begin: Style,
    cursor_at_end: bool,
    cursor_style: Style,
    tab_len: u8,
    mask: Option<char>,
    select_at_end: bool,
    select_style: Style,
}

impl<'a> LineHighlighter<'a> {
    pub fn new(
        line: &'a str,
        cursor_style: Style,
        tab_len: u8,
        mask: Option<char>,
        select_style: Style,
    ) -> Self {
        Self {
            line,
            spans: vec![],
            boundaries: vec![],
            style_begin: Style::default(),
            cursor_at_end: false,
            cursor_style,
            tab_len,
            mask,
            select_at_end: false,
            select_style,
        }
    }

    pub fn cursor_line(&mut self, cursor_col: usize, style: Style) {
        if let Some((start, c)) = self.line.char_indices().nth(cursor_col) {
            self.boundaries.push((Boundary::Cursor(self.cursor_style), start));
            self.boundaries.push((Boundary::End, start + c.len_utf8()));
        } else {
            self.cursor_at_end = true;
        }
        self.style_begin = style;
    }

    #[cfg(feature = "search")]
    pub fn search(&mut self, matches: impl Iterator<Item = (usize, usize)>, style: Style) {
        for (start, end) in matches {
            if start != end {
                self.boundaries.push((Boundary::Search(style), start));
                self.boundaries.push((Boundary::End, end));
            }
        }
    }

    pub fn selection(
        &mut self,
        current_row: usize,
        start_row: usize,
        start_off: usize,
        end_row: usize,
        end_off: usize,
    ) {
        let (start, end) = if current_row == start_row {
            if start_row == end_row {
                (start_off, end_off)
            } else {
                self.select_at_end = true;
                (start_off, self.line.len())
            }
        } else if current_row == end_row {
            (0, end_off)
        } else if start_row < current_row && current_row < end_row {
            self.select_at_end = true;
            (0, self.line.len())
        } else {
            return;
        };
        if start != end {
            self.boundaries.push((Boundary::Select(self.select_style), start));
            self.boundaries.push((Boundary::End, end));
        }
    }

    pub fn into_spans(self) -> Line<'a> {
        let Self {
            line,
            mut spans,
            mut boundaries,
            tab_len,
            style_begin,
            cursor_style,
            cursor_at_end,
            mask,
            select_at_end,
            select_style,
        } = self;
        let mut builder = DisplayTextBuilder::new(tab_len, mask);

        if boundaries.is_empty() {
            let built = builder.build(line);
            if !built.is_empty() {
                spans.push(Span::styled(built, style_begin));
            }
            if cursor_at_end {
                spans.push(Span::styled(" ", cursor_style));
            } else if select_at_end {
                spans.push(Span::styled(" ", select_style));
            }
            return Line::from(spans);
        }

        boundaries.sort_unstable_by(|(l, i), (r, j)| match i.cmp(j) {
            Ordering::Equal => l.cmp(r),
            o => o,
        });

        let mut style = style_begin;
        let mut start = 0;
        let mut stack = vec![];

        for (next_boundary, end) in boundaries {
            if start < end {
                spans.push(Span::styled(builder.build(&line[start..end]), style));
            }

            style = if let Some(s) = next_boundary.style() {
                stack.push(style);
                s
            } else {
                stack.pop().unwrap_or(style_begin)
            };
            start = end;
        }

        if start != line.len() {
            spans.push(Span::styled(builder.build(&line[start..]), style));
        }

        if cursor_at_end {
            spans.push(Span::styled(" ", cursor_style));
        } else if select_at_end {
            spans.push(Span::styled(" ", select_style));
        }

        Line::from(spans)
    }
}
