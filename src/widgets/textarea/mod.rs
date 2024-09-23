pub(super) mod behaviour {
    pub(super) mod cursor;
    pub(super) mod highlight;
    pub(super) mod input;
    pub(super) mod scroll;
    pub(super) mod util;
}

mod core;

pub use {
    behaviour::input::{Input, Key},
    core::{
        validation::{validators, ValidationResult},
        TextArea,
    },
};
