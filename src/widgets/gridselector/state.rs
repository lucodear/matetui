//! # Grid Selector State
//!
//! This module contains the [`GridSelectorState`] for the `GridSelector` widget.
//!
//! The state is used to keep track of the items, the selected item, and the hovered item and
//! encapsulates the navigation logic for the grid selector.

use ratatui::text::Text;

// If Text has a lifetime parameter, specify it in the implementation.
#[derive(Clone, Debug)]
pub struct GridItem(String);

impl GridItem {
    // accept both String and &str
    pub fn new<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        GridItem(value.into())
    }
}

// convert Label into &str
impl AsRef<str> for GridItem {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// convert Label into String
impl From<GridItem> for String {
    fn from(val: GridItem) -> Self {
        val.0.clone()
    }
}

// Specify the lifetime for the implementation
impl<'a> From<GridItem> for Text<'a> {
    fn from(val: GridItem) -> Self {
        Text::from(val.0)
    }
}

// implement a way to convert String into Label
impl From<String> for GridItem {
    fn from(value: String) -> Self {
        GridItem(value)
    }
}

// implement a way to convert &str into Label
impl<'a> From<&'a str> for GridItem {
    fn from(value: &'a str) -> Self {
        GridItem(value.to_string())
    }
}

/// State for the [`GridSelector`] widget.
///
/// This state is used to keep track of the items, the selected item, and the hovered item.
#[derive(Debug, Clone)]
pub struct GridSelectorState {
    pub items: Vec<GridItem>,
    pub selected: Option<usize>,
    pub hovered: Option<usize>,
    pub(crate) columns: usize,
}

impl GridSelectorState {
    /// Create a new [`GridSelectorState`] with the given items.
    pub fn new<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<GridItem>, // Accept anything that can be converted into Label
    {
        let items: Vec<GridItem> = items.into_iter().map(Into::into).collect();
        Self {
            items,
            selected: None,
            hovered: Some(0),
            columns: 5,
        }
    }

    /// builder method to set the number of columns
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    /// Get the selected item.
    pub fn selected(&self) -> Option<GridItem> {
        self.selected.map(|i| self.items[i].clone())
    }

    /// Get the index of the selected item.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Get the hovered item.
    pub fn hovered(&self) -> Option<GridItem> {
        self.hovered.map(|i| self.items[i].clone())
    }

    /// Move the hovered item right +1
    ///
    ///  Returns `true` if the hovered item was moved, `false` otherwise.
    pub fn move_right(&mut self) -> bool {
        self.hovered = if let Some(hovered) = self.hovered {
            let next = hovered + 1;
            if next < self.items.len() {
                Some(next)
            } else {
                Some(0)
            }
        } else {
            Some(0)
        };

        true
    }

    /// Move the hovered item left -1
    ///
    /// Returns `true` if the hovered item was moved,
    /// `false` otherwise.
    pub fn move_left(&mut self) -> bool {
        self.hovered = if let Some(hovered) = self.hovered {
            if hovered > 0 {
                Some(hovered - 1)
            } else {
                Some(self.items.len() - 1)
            }
        } else {
            Some(0)
        };
        true
    }

    /// Move the hovered item down by one row.
    ///
    /// Returns `true` if the hovered item was moved, `false` otherwise.
    pub fn move_down(&mut self) -> bool {
        if let Some(hovered) = self.hovered {
            let items_per_row = self.columns;
            let num_items = self.items.len();
            let current_row = hovered / items_per_row;
            let next_row_start = (current_row + 1) * items_per_row;
            let last_item_index = num_items - 1;

            // If we are in the last row, we can't go down
            if next_row_start > last_item_index {
                return false;
            }

            let mut next_index = std::cmp::min(hovered + items_per_row, last_item_index);

            // Handle the case where the next row has fewer items
            let next_row_count = std::cmp::min(items_per_row, last_item_index - next_row_start + 1);

            // check if both the next_row_count and the self.columns are odd numbers (3,5,7, etc)
            // and next_row_count is less than self.columns
            if next_row_count % 2 != 0 && items_per_row % 2 != 0 && next_row_count < items_per_row {
                let shift = (items_per_row - next_row_count) / 2;
                // if we are in the shifted range (left or right) of the row, adjust the hovered
                // index accordingly
                if hovered % items_per_row >= shift
                    && hovered % items_per_row < items_per_row - shift
                {
                    next_index = hovered + items_per_row - shift;
                } else {
                    // if we are in the left part of the shifted range, set next to the first item
                    // in the last row and if we are in the right part of the shifted range, set
                    // next to the last item in the last row
                    next_index = if hovered % items_per_row < shift {
                        next_row_start
                    } else {
                        last_item_index
                    };
                }
            }

            self.hovered = Some(std::cmp::min(next_index, last_item_index));
            return true;
        }

        false
    }

    /// Move the hovered item up by one row.
    ///
    /// Returns `true` if the hovered item was moved, `false` otherwise.
    pub fn move_up(&mut self) -> bool {
        if let Some(hovered) = self.hovered {
            let row_number = hovered / self.columns;

            // If we are in the first row, we can't go up
            if row_number == 0 {
                return false;
            }

            let mut next_index = hovered.saturating_sub(self.columns);

            // Handle case where the current index is in the last row
            // let is_last_row = hovered >= self.items.len().saturating_sub(self.columns);
            let last_row_start = (self.items.len() / self.columns) * self.columns;
            let is_last_row = hovered >= last_row_start;

            if is_last_row {
                let last_row_count = self.items.len() % self.columns;

                // If the last_row_count and self.columns are odd numbers (3,5,7, etc)
                // and last_row_count is less than self.columns we need to adjust the next index
                // to go to the cell just above the current hovered cell
                if last_row_count % 2 != 0 && self.columns % 2 != 0 && last_row_count < self.columns
                {
                    let shift = (self.columns - last_row_count) / 2;
                    next_index = hovered.saturating_sub(self.columns - shift);
                }
            }

            // Ensure next_index stays within bounds
            self.hovered = Some(std::cmp::min(next_index, self.items.len() - 1));
            return true;
        }

        false
    }

    /// Move the hovered item to the first item in the current row.
    ///
    /// Returns `true` if the hovered item was moved, `false` otherwise.
    pub fn move_to_row_start(&mut self) -> bool {
        if let Some(hovered) = self.hovered {
            let row_start = (hovered / self.columns) * self.columns;
            self.hovered = Some(row_start);
            true
        } else {
            false
        }
    }

    /// Move the hovered item to the last item in the current row.
    ///
    /// Returns `true` if the hovered item was moved, `false` otherwise.
    pub fn move_to_row_end(&mut self) -> bool {
        if let Some(hovered) = self.hovered {
            let row_end = std::cmp::min(
                (hovered / self.columns + 1) * self.columns - 1,
                self.items.len() - 1,
            );
            self.hovered = Some(row_end);
            true
        } else {
            false
        }
    }

    /// Select the hovered item.
    ///
    /// Select the hovered item. Returns `true` if the hovered item was selected, `false` otherwise.
    pub fn select(&mut self) -> bool {
        if let Some(hovered) = self.hovered {
            self.selected = Some(hovered);
            true
        } else {
            false
        }
    }
}
