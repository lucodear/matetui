use {
    super::{validation::ValidatorFn, TextArea},
    ratatui::{layout::Alignment, style::Style, widgets::Block},
};

impl<'a> TextArea<'a> {
    /// Set the style of textarea. By default, textarea is not styled.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the block of textarea. By default, no block is set.
    pub fn with_block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the placeholder text. The text is set in the textarea when no text is input. Setting a
    /// non-empty string `""` enables the placeholder. The default value is an empty string so the
    /// placeholder is disabled by default. To customize the text style, see
    /// [`TextArea::set_placeholder_style`].
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the style of the placeholder text. The default style is a dark gray text.
    pub fn with_placeholder_style(mut self, style: Style) -> Self {
        self.placeholder_style = style;
        self
    }

    /// Specify a character masking the text. All characters in the textarea will be replaced by
    /// this character. This API is useful for making a kind of credentials form such as a password
    /// input.
    pub fn with_mask_char(mut self, mask: Option<char>) -> Self {
        self.mask = mask;
        self
    }

    /// Set the style of cursor. By default, a cursor is rendered in the reversed color. Setting the
    /// same style as cursor line hides a cursor.
    pub fn with_cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    /// Set text alignment. When [`Alignment::Center`] or [`Alignment::Right`] is set, line number
    /// is automatically disabled because those alignments don't work well with line numbers.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_validations(
        mut self,
        validations: impl IntoIterator<
            Item = impl Fn(&str) -> Result<(), String> + Send + Sync + 'static,
        >,
    ) -> Self {
        self.validators.extend(validations.into_iter().map(ValidatorFn::new));
        self
    }
}
