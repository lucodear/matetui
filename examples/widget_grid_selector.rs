use std::io;

use {
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    matetui::widgets::gridselector::{GridItem, GridSelector, GridSelectorState},
    ratatui::{
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        prelude::CrosstermBackend,
        style::{Color, Modifier, Style, Stylize},
        widgets::Paragraph,
        Frame, Terminal,
    },
};

struct Item {
    name: String,
    emoji: String,
}

impl Item {
    fn new(name: &str, emoji: &str) -> Self {
        Item {
            name: name.to_string(),
            emoji: emoji.to_string(),
        }
    }
}

impl From<Item> for GridItem {
    fn from(val: Item) -> Self {
        GridItem::new(format!("{} {}", val.name, val.emoji))
    }
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // Run the draw loop
    run(&mut term)?;

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    Ok(())
}

fn run<B>(term: &mut Terminal<B>) -> io::Result<()>
where
    B: ratatui::backend::Backend,
{
    let items = vec![
        Item::new("feat", "✨"),
        Item::new("fix", "🐛"),
        Item::new("chore", "🧹"),
        Item::new("docs", "📚"),
        Item::new("style", "💅"),
        Item::new("refactor", "🔨"),
        Item::new("perf", "🐎"),
        Item::new("test", "🚨"),
        Item::new("build", "👷"),
        Item::new("ci", "🔧"),
        Item::new("revert", "⏪"),
        Item::new("release", "🚀"),
        Item::new("wip", "🚧"),
    ];

    let mut grid_state = GridSelectorState::new(items).columns(5);

    loop {
        term.draw(|f| draw(f, f.area(), &mut grid_state))?;

        match event::read()? {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => break,
                KeyEvent {
                    code: KeyCode::Right,
                    kind: KeyEventKind::Press,
                    ..
                } => grid_state.move_right(),
                KeyEvent {
                    code: KeyCode::Left,
                    kind: KeyEventKind::Press,
                    ..
                } => grid_state.move_left(),
                KeyEvent {
                    code: KeyCode::Up,
                    kind: KeyEventKind::Press,
                    ..
                } => grid_state.move_up(),
                KeyEvent {
                    code: KeyCode::Down,
                    kind: KeyEventKind::Press,
                    ..
                } => grid_state.move_down(),
                KeyEvent {
                    code: KeyCode::Home,
                    kind: KeyEventKind::Press,
                    ..
                } => grid_state.move_to_row_start(),
                KeyEvent {
                    code: KeyCode::End,
                    kind: KeyEventKind::Press,
                    ..
                } => grid_state.move_to_row_end(),
                // selection
                KeyEvent {
                    code: KeyCode::Enter,
                    kind: KeyEventKind::Press,
                    ..
                } => grid_state.select(),
                _ => false,
            },
            _ => false,
        };
    }

    Ok(())
}

fn draw(f: &mut Frame<'_>, area: Rect, state: &mut GridSelectorState) {
    // vertical layout with 2 rows
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(area);

    f.render_stateful_widget(
        GridSelector::default().with_selected_color(Color::Magenta),
        layout[0],
        state,
    );

    let selected = state.selected().unwrap_or(GridItem::new("None"));
    let hovered = state.hovered().unwrap_or(GridItem::new("None"));

    let ps = Paragraph::new(selected)
        .alignment(Alignment::Center)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Magenta));

    let ph = Paragraph::new(hovered)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Blue));

    f.render_widget(
        Paragraph::new("Hovered/Selected")
            .alignment(Alignment::Center)
            .add_modifier(Modifier::BOLD),
        layout[1],
    );
    f.render_widget(ph, layout[2]);
    f.render_widget(ps, layout[3]);
}
