use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// A switch widget
///
/// This widget is used to show visual confirmation of a boolean state
pub struct Switch {
    /// The state of the switch
    state: bool,
    /// The color of the "on" state (`Green` by default)
    color_on: Color,
    /// The color of the "off" state (`DarkGray` by default)
    color_off: Color,
    /// The color of the switch itself (`White` by default)
    color_switch: Color,
}

impl Switch {
    pub fn with_status(state: bool) -> Self {
        Switch {
            state,
            color_on: Color::Green,
            color_off: Color::DarkGray,
            color_switch: Color::White,
        }
    }

    pub fn with_color_on(mut self, color: Color) -> Self {
        self.color_on = color;
        self
    }

    pub fn with_color_off(mut self, color: Color) -> Self {
        self.color_off = color;
        self
    }

    pub fn with_color_switch(mut self, color: Color) -> Self {
        self.color_switch = color;
        self
    }

    pub fn get_layout(&self, area: Rect) -> (Rect, Rect) {
        let main = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(2)])
            .split(area);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Length(7), Constraint::Length(7)])
            .split(main[0]);

        (layout[0], layout[1])
    }

    fn get_left_color(&self) -> Color {
        if self.state {
            self.color_on
        } else {
            self.color_switch
        }
    }

    fn get_right_color(&self) -> Color {
        if self.state {
            self.color_switch
        } else {
            self.color_off
        }
    }
}

impl Widget for Switch {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (left, right) = self.get_layout(area);

        let left_block = Block::default().style(Style::default().bg(self.get_left_color()));
        let right_block = Block::default().style(Style::default().bg(self.get_right_color()));

        left_block.render(left, buf);
        right_block.render(right, buf);
    }
}
