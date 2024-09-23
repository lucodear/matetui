use {
    super::{GridSelector, GridSelectorState},
    ratatui::{
        buffer::Buffer,
        layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
        style::Style,
        widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
    },
    std::rc::Rc,
    unicode_width::UnicodeWidthStr,
};

impl StatefulWidget for GridSelector {
    type State = GridSelectorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut GridSelectorState) {
        let rows_layout = rows_layout(state, area);
        let largest_item = largest_item(state);

        for (i, row) in rows_layout.iter().enumerate() {
            let row_items = state.items.iter().skip(i * state.columns).take(state.columns);
            let columns_layout = columns_layout(row, row_items.len(), largest_item);

            for (j, item) in row_items.enumerate() {
                let main_index = i * state.columns + j;
                let color = self.get_color(main_index, state);

                let type_block =
                    Block::default().borders(Borders::ALL).border_style(Style::default().fg(color));

                Paragraph::new(item.clone())
                    .style(Style::default().fg(color))
                    .alignment(Alignment::Left)
                    .block(type_block)
                    .render(columns_layout[j], buf);
            }
        }
    }
}

fn rows_layout(state: &GridSelectorState, area: Rect) -> Rc<[Rect]> {
    let row_count = (state.items.len() as f32 / state.columns as f32).ceil() as usize;

    Layout::default()
        .direction(Direction::Vertical)
        .constraints((0..row_count).map(|_| Constraint::Length(3)).collect::<Vec<_>>())
        .spacing(0)
        .split(area)
}

fn columns_layout(row: &Rect, row_item_count: usize, largest_item: u16) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .constraints(
            (0..row_item_count).map(|_| Constraint::Length(largest_item + 3)).collect::<Vec<_>>(),
        )
        .split(*row)
}

fn largest_item(state: &GridSelectorState) -> u16 {
    state.items.iter().map(|item| UnicodeWidthStr::width(item.as_ref())).max().unwrap_or(0) as u16
}
