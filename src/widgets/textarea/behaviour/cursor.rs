use {
    super::util::{find_word_start_backward, find_word_start_forward},
    crate::widgets::textarea::core::widget::Viewport,
    std::cmp,
};

/// Specify how to move the cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorMove {
    /// Move cursor forward by one character. When the cursor is at the end of line, it moves to the
    /// head of next line.
    Forward,
    /// Move cursor backward by one character. When the cursor is at the head of line, it moves to
    /// the end of previous line.
    Back,
    /// Move cursor up by one line.
    Up,
    /// Move cursor down by one line.
    Down,
    /// Move cursor to the head of line. When the cursor is at the head of line, it moves to the end of previous line.
    Head,
    /// Move cursor to the end of line. When the cursor is at the end of line, it moves to the head of next line.
    End,
    /// Move cursor forward by one word. Word boundary appears at spaces, punctuations, and others. For example
    /// `fn foo(a)` consists of words `fn`, `foo`, `(`, `a`, `)`. When the cursor is at the end of line, it moves to the
    /// head of next line.
    WordForward,
    /// Move cursor backward by one word.  Word boundary appears at spaces, punctuations, and others. For example
    /// `fn foo(a)` consists of words `fn`, `foo`, `(`, `a`, `)`.When the cursor is at the head of line, it moves to
    /// the end of previous line.
    WordBack,
    /// Move cursor to keep it within the viewport. For example, when a viewport displays line 8 to line 16:
    ///
    /// - cursor at line 4 is moved to line 8
    /// - cursor at line 20 is moved to line 16
    /// - cursor at line 12 is not moved
    ///
    /// This is useful when you moved a cursor but you don't want to move the viewport.
    InViewport,
}

impl CursorMove {
    pub(crate) fn next_cursor(
        &self,
        (row, col): (usize, usize),
        lines: &[String],
        viewport: &Viewport,
    ) -> Option<(usize, usize)> {
        use CursorMove::*;

        fn fit_col(col: usize, line: &str) -> usize {
            cmp::min(col, line.chars().count())
        }

        match self {
            Forward if col >= lines[row].chars().count() => {
                (row + 1 < lines.len()).then(|| (row + 1, 0))
            }
            Forward => Some((row, col + 1)),
            Back if col == 0 => {
                let row = row.checked_sub(1)?;
                Some((row, lines[row].chars().count()))
            }
            Back => Some((row, col - 1)),
            Up => {
                let row = row.checked_sub(1)?;
                Some((row, fit_col(col, &lines[row])))
            }
            Down => Some((row + 1, fit_col(col, lines.get(row + 1)?))),
            Head => Some((row, 0)),
            End => Some((row, lines[row].chars().count())),
            WordForward => {
                if let Some(col) = find_word_start_forward(&lines[row], col) {
                    Some((row, col))
                } else if row + 1 < lines.len() {
                    Some((row + 1, 0))
                } else {
                    Some((row, lines[row].chars().count()))
                }
            }
            WordBack => {
                if let Some(col) = find_word_start_backward(&lines[row], col) {
                    Some((row, col))
                } else if row > 0 {
                    Some((row - 1, lines[row - 1].chars().count()))
                } else {
                    Some((row, 0))
                }
            }
            InViewport => {
                let (row_top, col_top, row_bottom, col_bottom) = viewport.position();

                let row = row.clamp(row_top as usize, row_bottom as usize);
                let row = cmp::min(row, lines.len() - 1);
                let col = col.clamp(col_top as usize, col_bottom as usize);
                let col = fit_col(col, &lines[row]);

                Some((row, col))
            }
        }
    }
}
