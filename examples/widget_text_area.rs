use {
    matetui::{
        ratatui::{
            backend::CrosstermBackend,
            crossterm::{
                event::{self},
                execute,
                terminal::{
                    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
                },
            },
            layout::{Constraint, Direction, Layout},
            Terminal,
        },
        widgets::textarea::{Input, Key, TextArea},
    },
    ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph},
    std::{cmp, io},
};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = TextArea::default().with_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Your name ")
            .padding(Padding::horizontal(1)),
    );

    loop {
        term.draw(|f| {
            const MIN_HEIGHT: usize = 1;
            let height = cmp::max(textarea.lines().len(), MIN_HEIGHT) as u16 + 2;
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(height), Constraint::Min(1)])
                .split(f.area());

            // Render the textarea
            f.render_widget(&textarea, layout[0]);
            f.render_widget(Paragraph::new("Press <Esc> to exit"), layout[1]);
        })?;
        match event::read()?.into() {
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('c'),
                shift: false,
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Enter,
                ctrl: false,
                shift: false,
                alt: false,
            } => break,
            input => {
                textarea.input(input);
            }
        }
    }

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;

    println!("Lines: {:?}", textarea.lines());
    Ok(())
}
