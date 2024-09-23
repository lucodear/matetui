use {
    super::events::Event,
    crossterm::{
        cursor,
        event::{
            DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
            Event as CrosstermEvent, KeyEventKind,
        },
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    futures::{FutureExt, StreamExt},
    ratatui::backend::CrosstermBackend as Backend,
    std::{
        ops::{Deref, DerefMut},
        time::Duration,
    },
    tokio::{
        sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
        task::JoinHandle,
    },
    tokio_util::sync::CancellationToken,
};

pub type IO = std::io::Stdout;
fn io() -> IO {
    std::io::stdout()
}
pub type Frame<'a> = ratatui::Frame<'a>;

/// The Tui struct represents a terminal user interface.
///
/// It encapsulates [ratatui::Terminal] adding extra functionality:
/// - [Tui::start] and [Tui::stop] to start and stop the event loop
/// - [Tui::enter] and [Tui::exit] to enter and exit the crossterm terminal
///   [raw mode](https://docs.rs/crossterm/0.28.1/crossterm/terminal/index.html#raw-mode)
/// - Mapping of crossterm events to [Event]s
/// - Emits [Event::Tick] and [Event::Render] events at a specified rate
pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<IO>>,
    pub task: JoinHandle<()>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub mouse: bool,
    pub paste: bool,
}

impl Tui {
    pub fn new() -> Result<Self, std::io::Error> {
        let tick_rate = 4.0;
        let frame_rate = 60.0;
        let terminal = ratatui::Terminal::new(Backend::new(io()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});
        let mouse = false;
        let paste = false;
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
            mouse,
            paste,
        })
    }

    /// Sets the tick rate for the Tui. The tick rate is the number of times per second that the
    /// Tui will emit a [Event::Tick] event. The default tick rate is 4 ticks per second.
    ///
    /// The tick is different from the render rate, which is the number of times per second that
    /// the application will be drawn to the screen. The tick rate is useful for updating the
    /// application state, performing calculations, run background tasks, and other operations that
    /// do not require a per-frame operation.
    ///
    /// Tick rate will usually be lower than the frame rate.
    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Sets the frame rate for the Tui. The frame rate is the number of times per second that the
    /// Tui will emit a [Event::Render] event. The default frame rate is 60 frames per second.
    ///
    /// The frame rate is the rate at which the application will be drawn to the screen (by calling
    /// the `draw` method of each component).
    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    /// Sets whether the Tui should capture mouse events. The default is false.
    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    /// Sets whether the Tui should capture paste events. The default is false.
    pub fn paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }

    /// Starts the Tui event loop.
    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancellation_token = self.cancellation_token.clone();
        let _event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            _event_tx.send(Event::Init).unwrap();
            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = _cancellation_token.cancelled() => {
                        println!("Tui task cancelled");
                        break;
                    }
                    maybe_event = crossterm_event => {
                        match maybe_event {
                        Some(Ok(evt)) => {
                            match evt {
                                CrosstermEvent::Key(key) => {
                                    if key.kind == KeyEventKind::Press {
                                        _event_tx.send(Event::Key(key)).unwrap();
                                    }
                                },
                                CrosstermEvent::Mouse(mouse) => {
                                    _event_tx.send(Event::Mouse(mouse)).unwrap();
                                },
                                CrosstermEvent::Resize(x, y) => {
                                    _event_tx.send(Event::Resize(x, y)).unwrap();
                                },
                                CrosstermEvent::FocusLost => {
                                    _event_tx.send(Event::FocusLost).unwrap();
                                },
                                CrosstermEvent::FocusGained => {
                                    _event_tx.send(Event::FocusGained).unwrap();
                                },
                                CrosstermEvent::Paste(s) => {
                                    _event_tx.send(Event::Paste(s)).unwrap();
                                },
                            }
                        }
                        Some(Err(_)) => {
                            _event_tx.send(Event::Error).unwrap();
                        }
                        None => {},
                        }
                    },
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap();
                    },
                    _ = render_delay => {
                        _event_tx.send(Event::Render).unwrap();
                    },
                }
            }
        });
    }

    /// Stops the Tui event loop.
    pub fn stop(&self) {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                break;
            }
        }
    }

    /// Enables cross-term raw mode and enters the alternate screen.
    pub fn enter(&mut self) -> Result<(), std::io::Error> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(io(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse {
            crossterm::execute!(io(), EnableMouseCapture)?;
        }
        if self.paste {
            crossterm::execute!(io(), EnableBracketedPaste)?;
        }
        self.start();
        Ok(())
    }

    /// Disables cross-term raw mode and exits the alternate screen.
    pub fn exit(&mut self) -> Result<(), std::io::Error> {
        self.stop();
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.paste {
                crossterm::execute!(io(), DisableBracketedPaste)?;
            }
            if self.mouse {
                crossterm::execute!(io(), DisableMouseCapture)?;
            }
            crossterm::execute!(io(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn suspend(&mut self) -> Result<(), std::io::Error> {
        self.exit()
    }

    pub fn resume(&mut self) -> Result<(), std::io::Error> {
        self.enter()
    }

    /// Returns the next event from the event channel.
    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<IO>>;

    fn deref(&self) -> &Self::Target {
        // deref Tui as Terminal
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // deref Tui as Terminal mutably
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        // Ensure that the terminal is cleaned up when the Tui is dropped
        self.exit().unwrap();
    }
}
