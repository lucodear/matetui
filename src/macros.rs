/// Creates a vector of components.
///
/// Each component will be converted into a `Box<dyn Component>`.
///
/// ## Example
///
/// ```rust
/// # use matetui::{components, component, Component};
/// # component! {
/// # struct FpsComponent {}
/// # }
/// #
/// # impl Component for FpsComponent {
/// #    fn draw(&mut self, _: &mut matetui::Frame<'_>, _: matetui::ratatui::prelude::Rect) {}
/// # }
///
/// let components = components![
///    FpsComponent::default()
///    //, AnotherComponent::default()
/// ];
#[macro_export]
macro_rules! components {
    ( $( $x:expr $( => $t:ty )* ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(
                    Box::new($x)
                        as Box<dyn matetui::Component $( $t + )* >
                );
            )*
            temp_vec
        }
    };
}

/// Creates a hashmap of children components.
///
/// Each child will be converted into a `Box<dyn Component>`.
///
/// ## Example
///
/// ```rust
/// # use matetui::{children, component, Component};
/// # component! {
/// # struct FpsComponent {}
/// # }
/// #
/// # impl Component for FpsComponent {
/// #    fn draw(&mut self, _: &mut matetui::Frame<'_>, _: matetui::ratatui::prelude::Rect) {}
/// # }
///
/// let children = children! {
///    "fps-counter" => FpsComponent::default()
/// };
/// ```
#[macro_export]
macro_rules! children {
    ( $( $name:expr => $value:expr ),* ) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert(
                    $name.to_string(),
                    Box::new($value) as Box<dyn matetui::Component>
                );
            )*
            map
        }
    };
}

/// #### component builder macro
///
/// The `component!` macro is used to define a new `Component` struct, expanding the struct
/// definition with the required fields and derives, and implementing the `ComponentAccesor` trait
/// for it.
///
/// ## Example
///
/// ```rust
/// use matetui::{component};
/// # use matetui::Component;
/// # component! {
/// #   struct FpsComponent {}
/// # }
/// #
/// # impl Component for FpsComponent {
/// #   fn draw(&mut self, _: &mut matetui::Frame<'_>, _: matetui::ratatui::prelude::Rect) {}
/// # }
/// # impl Component for MainComponent {
/// #   fn draw(&mut self, _: &mut matetui::Frame<'_>, _: matetui::ratatui::prelude::Rect) {}
/// # }
///
/// component! (
///   // mandatory struct definition
///   pub struct MainComponent {
///       counter: u32
///   },
///
///   // optional children definition
///   children => {
///       "fps-counter" => FpsComponent::default()
///   }
/// );
#[macro_export]
macro_rules! component {
    // Entry point: struct definition without children
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident { $($fieldname:ident: $ty:ty),* $(,)? }
    ) => {
        // Call the inner macro with an empty children section
        component! {
            $(#[$outer])*
            $vis struct $name { $($fieldname: $ty),* },
            children => {}
        }
    };

    // Entry point: struct definition with children
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident { $($fieldname:ident: $ty:ty),* $(,)? },
        children => {
            $($childname:literal => $childval:expr),* $(,)?
        }
    ) => {
        // Expand the struct with fields and children
        $(#[$outer])*
        $vis struct $name {
            is_active: bool,
            action_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
            children: matetui::Children,
            $($fieldname: $ty),*
        }

        // Implement the default trait for the struct
        impl Default for $name {
            fn default() -> Self {
                Self {
                    is_active: false,
                    action_sender: None,
                    children: matetui::children!( $($childname => $childval),* ),
                    $($fieldname: Default::default()),*
                }
            }
        }

        // Implement the ComponentAccessors trait
        impl matetui::ComponentAccessors for $name {
            fn name(&self) -> String {
                stringify!($name).to_string()
            }
            fn is_active(&self) -> bool {
                self.is_active
            }
            fn set_active(&mut self, active: bool) {
                self.is_active = active;
            }
            fn register_action_handler(&mut self, tx: tokio::sync::mpsc::UnboundedSender<String>) {
                self.action_sender = Some(tx.clone());
            }
            fn send(&self, action: &str) {
                if let Some(tx) = &self.action_sender {
                    tx.send(action.to_string()).unwrap();
                }
            }
            fn send_action(&self, action: matetui::Action) {
                if let Some(tx) = &self.action_sender {
                    tx.send(action.to_string()).unwrap();
                }
            }
            fn as_active(mut self) -> Self {
                self.set_active(true);
                self
            }
            fn get_children(&mut self) -> Option<&mut matetui::Children> {
                Some(&mut self.children)
            }
        }
    };
}

/// Creates an array of keybindings.
///
/// Each action will be converted into an `ActionKind`.
///
/// This macro accepts two syntaxes:
///
/// 1. `<key> => <action>` syntax:
///
/// ```rust
/// # use matetui::{kb, Action};
/// let keybindings = kb![
///     "<q>" => Action::Quit,
///     "<d>" => "app:drink-mate"
/// ];
/// ```
///
/// 2. `(<key>, <action>)` syntax:
///
/// ```rust
/// # use matetui::{kb, Action};
/// let keybindings = kb![
///     ("<q>", Action::Quit),
///     ("<d>", "app:drink-mate")
/// ];
/// ```
///
/// Each action will be converted into an `ActionKind`.
#[macro_export]
macro_rules! kb {
    // Accepts "<key>" => <action> syntax
    ($($key:expr => $action:expr),* $(,)?) => {
        [
            $(($key, $crate::ActionKind::from($action))),*
        ]
    };

    // Accepts ("<key>", <action>) syntax
    ($(($key:expr, $action:expr)),* $(,)?) => {
        [
            $(($key, $crate::ActionKind::from($action))),*
        ]
    };
}
