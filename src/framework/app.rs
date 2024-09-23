use {
    super::{
        component::Component,
        events::{Action, ActionKind, Event},
        keyboard::KeyBindings,
        tui::Tui,
    },
    crossterm::event::{KeyCode, KeyEvent},
    std::str::FromStr,
    thiserror::Error,
    tokio::sync::mpsc::{
        self,
        error::{SendError, TryRecvError},
    },
};

#[derive(Error, Debug)]
pub enum MatetuiError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("error sending message: {0}")]
    SendError(#[from] SendError<String>),
}

pub struct App {
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: Vec<Box<dyn Component>>,
    pub should_quit: bool,
    // pub should_suspend: bool,
    pub keybindings: KeyBindings,
    pub last_tick_key_events: Vec<KeyEvent>,
    pub mouse: bool,
    action_tx: mpsc::UnboundedSender<String>,
    action_rx: mpsc::UnboundedReceiver<String>,
}

impl Default for App {
    fn default() -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<String>();
        Self {
            last_tick_key_events: Vec::default(),
            keybindings: KeyBindings::default(),
            components: Vec::new(),
            frame_rate: 4.into(),
            tick_rate: 1.into(),
            should_quit: false,
            // should_suspend: false,
            mouse: false,
            action_tx,
            action_rx,
        }
    }
}

impl App {
    // pub fn with_keybindings<const N: usize>(mut self, kb: [(&str, &str); N]) -> Self
    pub fn new<const N: usize>(kb: [(&str, &str); N], components: Vec<Box<dyn Component>>) -> Self {
        let keybindings = KeyBindings::new(kb);

        Self {
            keybindings,
            components,
            ..Self::default()
        }
    }

    /// Set the components
    pub fn with_components(mut self, components: Vec<Box<dyn Component>>) -> Self {
        self.components = components;
        self
    }

    // Set the keybindings
    pub fn with_keybindings<const N: usize>(
        mut self,
        kb: [(&str, impl Into<ActionKind>); N],
    ) -> Self {
        self.keybindings = KeyBindings::new(kb);
        self
    }

    /// Set the tick rate
    pub fn with_tick_rate(mut self, tick_rate: impl Into<f64>) -> Self {
        self.tick_rate = tick_rate.into();
        self
    }

    /// Set the frame rate
    pub fn with_frame_rate(mut self, frame_rate: impl Into<f64>) -> Self {
        self.frame_rate = frame_rate.into();
        self
    }

    fn send(&self, action: Action) -> Result<(), MatetuiError> {
        match action {
            Action::AppAction(cmd) => self.action_tx.send(cmd)?,
            Action::Key(key) => self.action_tx.send(key)?,
            action => self.action_tx.send(action.to_string())?,
        };
        Ok(())
    }

    fn try_recv(&mut self) -> Result<String, TryRecvError> {
        self.action_rx.try_recv()
    }

    pub async fn run(&mut self) -> Result<(), MatetuiError> {
        let mut tui = Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);

        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone());
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?);
        }

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    // Event::Resize(x, y) => self.send(Action::Resize(x, y))?,
                    Event::Render => self.send(Action::Render)?,
                    Event::Tick => self.send(Action::Tick)?,
                    Event::Quit => self.send(Action::Quit)?,
                    Event::Key(key) => {
                        if let Some(action) = self.keybindings.get(&[key]) {
                            self.send(action.clone())?;
                        } else {
                            // If the key was not handled as a single key action,
                            // then consider it for multi-key combinations.
                            self.last_tick_key_events.push(key);

                            // Check for multi-key combinations
                            if let Some(action) = self.keybindings.get(&self.last_tick_key_events) {
                                self.send(action.clone())?;
                            }
                        }

                        // send the key event as simple key event too (not as action) if it's a
                        // single alphanumeric char key
                        if let KeyCode::Char(c) = key.code {
                            if c.is_alphanumeric() {
                                self.send(Action::Key(c.to_string()))?;
                            }
                        }
                    }
                    _ => {}
                }
                let mut actions = Vec::new();

                for component in self.components.iter_mut() {
                    if component.is_active() {
                        let component_actions = component.handle_events(Some(e.clone()));
                        actions.extend(component_actions);
                    }
                }

                for action in actions {
                    self.send(action)?;
                }
            }

            while let Ok(action) = self.try_recv() {
                let enum_action = Action::from_str(&action).ok();
                if let Some(a) = enum_action {
                    match a {
                        Action::Quit => self.should_quit = true,
                        Action::Render => {
                            tui.draw(|f| {
                                for component in self.components.iter_mut() {
                                    if component.is_active() {
                                        component.draw(f, f.area());
                                    }
                                }
                            })?;
                        }
                        Action::Tick => {
                            self.last_tick_key_events.drain(..);
                        }

                        // Action::Resize(w, h) => {
                        //     tui.resize(Rect::new(0, 0, w, h))?;
                        //     tui.draw(|f| {
                        //         for component in self.components.iter_mut() {
                        //             if component.is_active() {
                        //                 component.draw(f, f.area());
                        //             }
                        //         }
                        //     })?;
                        // }
                        _ => {}
                    }

                    for component in self.components.iter_mut() {
                        if component.is_active() {
                            component.update(a.clone());
                        }
                    }
                } else {
                    // unrecognized action, might be a custom component action
                    // send it to all components as a raw string
                    for component in self.components.iter_mut() {
                        if component.is_active() {
                            component.receive_message(action.clone());
                        }
                    }
                }
            }

            if self.should_quit {
                tui.stop();
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
