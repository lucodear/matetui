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
            layout::{Constraint, Layout},
            Terminal,
        },
        widgets::{
            switch::Switch,
            textarea::{Input, Key},
        },
    },
    ratatui::layout::Flex,
    std::io,
};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;
    let mut switch_state = false;

    loop {
        term.draw(|f| {
            // Render the textarea
            let switch =
                Switch::with_status(switch_state).with_color_on(ratatui::style::Color::Green);

            let [horiz] = Layout::horizontal([Constraint::Percentage(100)])
                .flex(Flex::Center)
                .areas(f.area());

            let [verti] = Layout::vertical([Constraint::Length(2)]).flex(Flex::Center).areas(horiz);

            let [centered] =
                Layout::horizontal([Constraint::Length(14)]).flex(Flex::Center).areas(verti);

            f.render_widget(switch, centered);
        })?;
        match event::read()?.into() {
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('c'),
                shift: false,
                ctrl: true,
                alt: false,
            } => break,
            Input {
                key: Key::Enter,
                ctrl: false,
                shift: false,
                alt: false,
            }
            | Input {
                key: Key::Char(' '),
                ..
            } => {
                switch_state = !switch_state;
            }
            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;

    Ok(())
}
