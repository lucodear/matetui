use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Backend-agnostic key input kind.
///
/// This type is marked as `#[non_exhaustive]` since more keys may be supported in the future.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, Default)]
pub enum Key {
    /// Normal letter key input
    Char(char),
    /// F1, F2, F3, ... keys
    // F(u8),
    /// Backspace key
    Backspace,
    /// Enter or return key
    Enter,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Tab key
    Tab,
    /// Delete key
    Delete,
    /// Home key
    Home,
    /// End key
    End,
    /// Escape key
    Esc,
    /// Copy key. This key is supported by termwiz only
    Copy,
    /// Cut key. This key is supported by termwiz only
    Cut,
    /// Paste key. This key is supported by termwiz only
    Paste,
    /// An invalid key input (this key is always ignored by [`TextArea`](crate::TextArea))
    #[default]
    Null,
}

/// Backend-agnostic key input type.
///
/// When `crossterm`, `termion`, `termwiz` features are enabled, converting respective key input types into this
/// `Input` type is defined.
/// ```no_run
/// use tui_textarea::{TextArea, Input, Key};
/// use crossterm::event::{Event, read};
///
/// let event = read().unwrap();
///
/// // `Input::from` can convert backend-native event into `Input`
/// let input = Input::from(event.clone());
/// // or `Into::into`
/// let input: Input = event.clone().into();
/// // Conversion from `KeyEvent` value is also available
/// if let Event::Key(key) = event {
///     let input = Input::from(key);
/// }
/// ```
///
/// Creating `Input` instance directly can cause backend-agnostic input as follows.
///
/// ```
/// use tui_textarea::{TextArea, Input, Key};
///
/// let mut textarea = TextArea::default();
///
/// // Input Ctrl+A
/// textarea.input(Input {
///     key: Key::Char('a'),
///     ctrl: true,
///     alt: false,
///     shift: false,
/// });
/// ```
#[derive(Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct Input {
    /// Typed key.
    pub key: Key,
    /// Ctrl modifier key. `true` means Ctrl key was pressed.
    pub ctrl: bool,
    /// Alt modifier key. `true` means Alt key was pressed.
    pub alt: bool,
    /// Shift modifier key. `true` means Shift key was pressed.
    pub shift: bool,
}

impl Input {
    /// Returns Some(char) if the Input is a single char without any modifiers (except Shift for
    /// case) and None otherwise.
    pub fn maybe_char(&self) -> Option<char> {
        match self {
            Input {
                key: Key::Char(c),
                ctrl: false,
                alt: false,
                ..
            } => Some(*c),
            _ => None,
        }
    }

    /// Returns `true` if the Input represents a new line, except is caused by Enter key without
    /// any modifiers. Useful for using Enter as a confirm key instead of a new line key.
    /// - Ctrl+Enter key
    /// - Alt+Enter key
    /// - Shift+Enter key
    /// - Char is \n or \r
    #[inline]
    pub fn is_newline_except_enter(&self) -> bool {
        matches!(
            self,
            Input {
                key: Key::Char('\n' | '\r'),
                ctrl: false,
                alt: false,
                ..
            } | Input {
                key: Key::Enter,
                ctrl: true,
                ..
            } | Input {
                key: Key::Enter,
                alt: true,
                ..
            } | Input {
                key: Key::Enter,
                shift: true,
                ..
            }
        )
    }

    /// Returns `true` if the Input represents a new line (including Enter key).
    /// - Enter key
    ///
    /// + all conditions of `is_newline_except_enter`
    #[inline]
    pub fn is_newline(&self) -> bool {
        self.is_newline_except_enter() || self.key == Key::Enter
    }

    /// Returns `true` if the Input is a single char without any modifiers (except Shift for upper
    /// case).
    #[inline]
    pub fn is_char(&self) -> bool {
        matches!(
            self,
            Input {
                key: Key::Char(_),
                ctrl: false,
                alt: false,
                ..
            }
        )
    }

    /// Returns `true` if the Input is a Tab
    #[inline]
    pub fn is_tab(&self) -> bool {
        self.key == Key::Tab && !self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is Backspace
    #[inline]
    pub fn is_backspace(&self) -> bool {
        self.key == Key::Backspace && !self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is Delete
    #[inline]
    pub fn is_delete(&self) -> bool {
        self.key == Key::Delete && !self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is key down arrow
    #[inline]
    pub fn is_down(&self) -> bool {
        self.key == Key::Down && !self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is key up arrow
    #[inline]
    pub fn is_up(&self) -> bool {
        self.key == Key::Up && !self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is key left arrow
    #[inline]
    pub fn is_left(&self) -> bool {
        self.key == Key::Left && !self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is key right arrow
    #[inline]
    pub fn is_right(&self) -> bool {
        self.key == Key::Right && !self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is key Home
    #[inline]
    pub fn is_home(&self) -> bool {
        self.key == Key::Home
    }

    /// Returns `true` if the Input is key End
    #[inline]
    pub fn is_end(&self) -> bool {
        self.key == Key::End
    }

    /// Returns `true` if the Input is ctrl+left
    #[inline]
    pub fn is_ctrl_left(&self) -> bool {
        self.key == Key::Left && self.ctrl && !self.alt
    }

    /// Returns `true` if the Input is ctrl+right
    #[inline]
    pub fn is_ctrl_right(&self) -> bool {
        self.key == Key::Right && self.ctrl && !self.alt
    }

    /// Returns a string representing the kind of key input.
    /// e.g ":delete", ":backspace", ":tab", ":enter", "char"
    /// or empty string if the key is null.
    /// uses the is_* methods to determine the kind of key input.
    pub fn kind(&self) -> &str {
        match self {
            i if i.is_delete() => ":delete",
            i if i.is_backspace() => ":backspace",
            i if i.is_tab() => ":tab",
            i if i.is_newline_except_enter() => ":non-enter-newline",
            i if i.is_newline() => ":newline",
            i if i.is_char() => ":char",
            i if i.is_down() => ":down",
            i if i.is_up() => ":up",
            i if i.is_left() => ":left",
            i if i.is_right() => ":right",
            i if i.is_home() => ":home",
            i if i.is_end() => ":end",
            i if i.is_ctrl_left() => ":word-left",
            i if i.is_ctrl_right() => ":word-right",
            _ => "",
        }
    }
}

impl From<Event> for Input {
    /// Convert [`crossterm::event::Event`] into [`Input`].
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => Self::from(key),
            _ => Self::default(),
        }
    }
}

impl From<KeyCode> for Key {
    /// Convert [`crossterm::event::KeyCode`] into [`Key`].
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Tab => Key::Tab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::Esc => Key::Esc,
            _ => Key::Null,
        }
    }
}

impl From<KeyEvent> for Input {
    /// Convert [`crossterm::event::KeyEvent`] into [`Input`].
    fn from(key: KeyEvent) -> Self {
        if key.kind == KeyEventKind::Release {
            // On Windows or when `crossterm::event::PushKeyboardEnhancementFlags` is set,
            // key release event can be reported. Ignore it. (#14)
            return Self::default();
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let key = Key::from(key.code);

        Self {
            key,
            ctrl,
            alt,
            shift,
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crossterm::event::KeyEventState};

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn key_to_input() {
        for (from, to) in [
            (
                key_event(KeyCode::Char('a'), KeyModifiers::empty()),
                input(Key::Char('a'), false, false, false),
            ),
            (
                key_event(KeyCode::Enter, KeyModifiers::empty()),
                input(Key::Enter, false, false, false),
            ),
            (
                key_event(KeyCode::Left, KeyModifiers::CONTROL),
                input(Key::Left, true, false, false),
            ),
            (
                key_event(KeyCode::Right, KeyModifiers::SHIFT),
                input(Key::Right, false, false, true),
            ),
            (
                key_event(KeyCode::Home, KeyModifiers::ALT),
                input(Key::Home, false, true, false),
            ),
            (
                key_event(KeyCode::NumLock, KeyModifiers::CONTROL),
                input(Key::Null, true, false, false),
            ),
        ] {
            assert_eq!(Input::from(from), to, "{:?} -> {:?}", from, to);
        }
    }

    #[test]
    fn event_to_input() {
        for (from, to) in [
            (
                Event::Key(key_event(KeyCode::Char('a'), KeyModifiers::empty())),
                input(Key::Char('a'), false, false, false),
            ),
            (Event::FocusGained, input(Key::Null, false, false, false)),
        ] {
            assert_eq!(Input::from(from.clone()), to, "{:?} -> {:?}", from, to);
        }
    }

    // Regression for https://github.com/rhysd/tui-textarea/issues/14
    #[test]
    fn ignore_key_release_event() {
        let mut from = key_event(KeyCode::Char('a'), KeyModifiers::empty());
        from.kind = KeyEventKind::Release;
        let to = input(Key::Null, false, false, false);
        assert_eq!(Input::from(from), to, "{:?} -> {:?}", from, to);
    }

    #[allow(dead_code)]
    pub(crate) fn input(key: Key, ctrl: bool, alt: bool, shift: bool) -> Input {
        Input {
            key,
            ctrl,
            alt,
            shift,
        }
    }
}
