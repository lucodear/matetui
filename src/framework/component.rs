use {
    super::{
        events::{Action, Event},
        tui::Frame,
    },
    crossterm::event::{KeyEvent, MouseEvent},
    downcast_rs::{impl_downcast, Downcast},
    ratatui::layout::{Rect, Size},
    std::collections::HashMap,
    tokio::sync::mpsc::UnboundedSender,
};

pub type Children = HashMap<String, Box<dyn Component>>;

// #[derive(Error, Debug)]
// pub enum ComponentError {
//     #[error("Component error: {0}")]
//     ComponentError(String),
// }

// type Result<T> = std::result::Result<T, ComponentError>;

// TODO: create a component! macro to simplify the creation of components, adding the boilerplate
// code, similar to what we do with the `rustler!` macro in the `rustler` crate.

/// `Component` is a trait that represents a visual and interactive element of the user interface.
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events,
/// update state, and be rendered on the screen.
pub trait Component: Downcast + ComponentAccessors {
    /// Initialize the component with a specified area if necessary.
    /// By default, this method will pass the initialization to the children. If you want to
    /// override this method, you will lose calling the children's `init` method. That's why we
    /// provide a helper function to initialize the children. Something like the following is
    /// recommended:
    ///
    /// ```rust
    /// fn init(&mut self, area: Rect) -> Result<()> {
    ///   // Do something with the area
    ///
    ///   // Initialize the children
    ///   init_children(self, area)
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area to initialize the component within.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    #[allow(unused)]
    fn init(&mut self, area: Size) {
        init_children(self, area)
    }

    /// Handle incoming events and produce actions if necessary.
    /// In most cases, you should avoid overriding this method, as it will handle the children's
    /// events and also rerout actions to `self.handle_key_events` and `self.handle_mouse_events`.
    ///
    /// In most cases, you might want to implement those two methods instead of this one.
    ///
    /// # Arguments
    ///
    /// * `event` - An optional event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_events(&mut self, event: Option<Event>) -> Vec<Action> {
        if self.is_active() {
            let mut actions = vec![];

            let action = match event {
                Some(Event::Key(key_event)) => self.handle_key_events(key_event),
                Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event),
                Some(Event::Tick) => self.handle_tick_event(),
                Some(Event::Render) => self.handle_frame_event(),
                Some(Event::Paste(ref event)) => self.handle_paste_event(event.clone()),
                _ => None,
            };

            if let Some(action) = action {
                actions.push(action);
            }

            if let Some(children) = self.get_children() {
                for child in children.values_mut() {
                    if child.is_active() {
                        let child_actions = child.handle_events(event.clone());
                        actions.extend(child_actions);
                    }
                }
            }

            actions
        } else {
            vec![]
        }
    }

    /// Handle key events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `key` - A key event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Option<Action> {
        None
    }

    /// Handle mouse events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `mouse` - A mouse event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Option<Action> {
        None
    }

    /// Handle Tick events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `tick` - A tick event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_tick_event(&mut self) -> Option<Action> {
        None
    }

    /// Handle frame events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `tick` - A tick event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_frame_event(&mut self) -> Option<Action> {
        None
    }

    /// Handle paste events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `message` - A string message to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_paste_event(&mut self, message: String) -> Option<Action> {
        None
    }

    /// Update the state of the component based on a received action. If you want to override this
    /// method, you will lose calling the children's update methods. That's why we provide a helper
    /// function to update the children's state. Something like the following is recommended:
    ///
    /// ```rust
    /// fn update(&mut self, action: Action) -> Result<()> {
    ///   // Do something with the action
    ///
    ///   // Update the children
    ///   update_children(self, action)
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) {
        update_children(self, action)
    }

    /// Receive a custom message, probably from another component.
    /// If you want to override this method, you will lose calling the children's `receive_message`
    /// method. That's why we provide a helper function to pass the message. Something like the
    /// following is recommended:
    ///
    /// ```rust
    /// fn receive_message(&mut self, message: String) -> Result<()> {
    ///   // Do something with the message
    ///
    ///   // Pass the message to the children
    ///   pass_message_to_children(self, message)
    /// }
    /// ```
    /// # Arguments
    ///
    /// * `message` - A string message to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    #[allow(unused_variables)]
    fn receive_message(&mut self, message: String) {
        pass_message_to_children(self, message)
    }

    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect);

    /// Get a child component by name as a mutable reference.
    ///
    /// # Arguments
    /// * `name` - The name of the child component.
    ///
    /// # Returns
    /// * `Option<&mut Box<dyn Component>>` - A mutable reference to the child component or none.
    fn child_mut(&mut self, name: &str) -> Option<&mut Box<dyn Component>> {
        if let Some(children) = self.get_children() {
            children.get_mut(name)
        } else {
            None
        }
    }

    /// Get a child component by name as an immutable reference.
    ///
    /// # Arguments
    /// * `name` - The name of the child component.
    ///
    /// # Returns
    /// * `Option<&Box<dyn Component>>` - A reference to the child component or none.
    #[allow(clippy::borrowed_box)]
    fn child(&mut self, name: &str) -> Option<&Box<dyn Component>> {
        if let Some(children) = self.get_children() {
            children.get(name)
        } else {
            None
        }
    }

    // /// gets the active state of the component.
    // ///
    // /// # Returns
    // /// * `bool` - The active state of the component.
    // fn is_active(&self) -> bool {
    //     true
    // }

    // /// Set the active state of the component.
    // ///
    // /// # Arguments
    // /// * `active` - The active state of the component.
    // fn set_active(&mut self, active: bool) {
    //     set_active_on_children(self, active);
    // }
}

impl_downcast!(Component);

/// Update the children's state based on a received action.
///
/// This helper function is used to update the children's state based on a received action. It was
/// created to allow to easily override the default `update` method of a component implementation
/// and still be able to call the children's `update` method.
pub fn update_children<T: Component + ?Sized>(this: &mut T, action: Action) {
    if this.is_active() {
        if let Some(children) = this.get_children() {
            for child in children.values_mut() {
                if child.is_active() {
                    child.update(action.clone());
                }
            }
        }
    }
}

/// Pass a message to the children of a component.
///
/// This helper function is used to pass a message to the children of a component. It was created
/// to allow to easily override the default `receive_message` method of a component implementation
/// and still be able pass the call to the children's `receive_message` method.
pub fn pass_message_to_children<T: Component + ?Sized>(this: &mut T, message: String) {
    if this.is_active() {
        if let Some(children) = this.get_children() {
            for child in children.values_mut() {
                if child.is_active() {
                    child.receive_message(message.clone());
                }
            }
        }
    }
}

/// Set active/inactive to the children of a component.
///
/// This helper function is used to set active/inactive to the children of a component. It was
/// created to allow to easily implement the default `set_active` method of a component
/// implementation and be able to call the children's `set_active` method.
pub fn set_active_on_children<T: Component + ?Sized>(this: &mut T, active: bool) {
    if let Some(children) = this.get_children() {
        for child in children.values_mut() {
            child.set_active(active);
        }
    }
}

/// Initialize the children of a component.
///
/// This helper function is used to initialize the children of a component. It was created to
/// allow to easily override the default `init` method of a component implementation and still be
/// able to call the children's `init` method.
pub fn init_children<T: Component + ?Sized>(this: &mut T, area: Size) {
    if let Some(children) = this.get_children() {
        for child in children.values_mut() {
            child.init(area);
        }
    }
}

/// Pass the action handler to the children of a component.
///
/// This helper function is used to pass the action handler to the children of a component. It was
/// created to allow to easily override the default `register_action_handler` method of a component
/// implementation and still be able to call the children's `register_action_handler` method.
pub fn pass_action_handler_to_children<T: Component + ?Sized>(
    this: &mut T,
    tx: UnboundedSender<String>,
) {
    if let Some(children) = this.get_children() {
        for child in children.values_mut() {
            child.register_action_handler(tx.clone());
        }
    }
}

/// Get a child downcasted to a specific type by name as a mutable reference.
///
/// # Arguments
/// * `name` - The name of the child component.
///
/// # Returns
/// * `Option<&mut T>` - A mutable reference to the child component or none.
pub fn child_downcast_mut<'a, CastTo: Component, This: Component + ?Sized>(
    this: &'a mut This,
    name: &str,
) -> Option<&'a mut CastTo> {
    if let Some(child) = this.child_mut(name) {
        child.downcast_mut::<CastTo>()
    } else {
        None
    }
}

/// Get a child downcasted to a specific type by name as an immutable reference.
///
/// # Arguments
/// * `name` - The name of the child component.
///
/// # Returns
/// * `Option<&T>` - A reference to the child component or none.
pub fn child_downcast<'a, CastTo: Component, This: Component + ?Sized>(
    this: &'a mut This,
    name: &str,
) -> Option<&'a CastTo> {
    if let Some(child) = this.child(name) {
        child.downcast_ref::<CastTo>()
    } else {
        None
    }
}

pub trait ComponentAccessors {
    // #region fields g&s

    /// returns the name of the component
    fn name(&self) -> String;

    /// returns the active state of the component
    fn is_active(&self) -> bool;

    /// sets the active state of the component
    fn set_active(&mut self, active: bool);

    /// registers an action handler that can send actions for processing if necessary
    fn register_action_handler(&mut self, tx: UnboundedSender<String>);

    /// send a message to through the action handler bus
    fn send(&self, action: &str);

    /// send a message to through the action handler bus
    fn send_action(&self, action: Action);

    // create a Component as default and active
    fn as_active(self) -> Self
    where
        Self: Sized;

    /// Get all child components. This is necessary if the component has children, as will be
    /// used by other functions to have knowledge of the children.
    ///
    /// # Attributes
    /// * `children`: `HashMap<String, Box<dyn Component>>` - All child components.
    ///
    /// # Returns
    ///
    /// * `Vec[&mut Box<dyn Component>]` - A vector of mutable references to the child components.
    fn get_children(&mut self) -> Option<&mut Children>;

    // #endregion
}
