//! {`GridSelector`} widget
//!
//! This module contains the [`BoxSelector`] ratatui widget.
//!
//! The grid selector is a stateful widget that allows the user to select an item from a list of
//! items displayed in a grid. The user can navigate the grid with the arrow keys, and select the
//! currently hovered item with the `Enter` key.
//!
//! The [`GridSelector`] widget uses a [`GridSelectorState`] to be able to keep its state between
//! renders.

use {super::GridSelectorState, ratatui::style::Color};

pub struct GridSelector {
    color: Color,
    hovered_color: Color,
    selected_color: Color,
}

impl Default for GridSelector {
    fn default() -> Self {
        Self {
            color: Color::Reset,
            hovered_color: Color::Blue,
            selected_color: Color::Green,
        }
    }
}

// imlementation of the build pattern for the GridSelector to set the colors

impl GridSelector {
    /// Set the color of the items in the grid.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the color of the hovered item in the grid.
    pub fn with_hovered_color(mut self, color: Color) -> Self {
        self.hovered_color = color;
        self
    }

    /// Set the color of the selected item in the grid.
    pub fn with_selected_color(mut self, color: Color) -> Self {
        self.selected_color = color;
        self
    }

    pub(crate) fn get_color(&self, for_idx: usize, state: &GridSelectorState) -> Color {
        let mut color = self.color;

        if let Some(hovered_idx) = state.hovered {
            if for_idx == hovered_idx {
                color = self.hovered_color;
            }
        }

        if let Some(selected_index) = state.selected {
            if for_idx == selected_index {
                color = self.selected_color;
            }
        }

        color
    }
}
