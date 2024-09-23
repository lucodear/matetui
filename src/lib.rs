#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/lucodear/matetui/master/.github/media/logo.svg"
)]

pub mod macros;

mod framework {
    pub mod app;
    pub mod component;
    pub mod events;
    pub mod keyboard;
    pub mod tui;
}

pub use framework::{
    app::App,
    component::{child_downcast, child_downcast_mut, Children, Component, ComponentAccessors},
    events::{Action, ActionKind, Event},
    keyboard::KeyBindings,
    tui::{Frame, Tui, IO},
};

pub mod utils {
    pub mod keyboard {
        pub use super::super::framework::keyboard::{key_event_to_string, parse_key_sequence};
    }
}

#[cfg(feature = "widget-gridselector")]
#[cfg(feature = "widget-textarea")]
#[cfg(feature = "widget-switch")]
pub mod widgets {
    #[cfg(feature = "widget-gridselector")]
    pub mod gridselector {
        mod selector;
        mod state;
        mod widget;

        pub use {selector::*, state::*};
    }

    #[cfg(feature = "widget-textarea")]
    pub mod textarea;

    #[cfg(feature = "widget-switch")]
    pub mod switch {
        mod widget;
        pub use widget::*;
    }
}

// re-export ratatui
pub mod ratatui {
    pub use ratatui::*;
}
