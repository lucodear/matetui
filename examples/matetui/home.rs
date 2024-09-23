use {
    crate::fps,
    matetui::{
        component,
        ratatui::{
            prelude::{Constraint, Layout, Line, Rect},
            widgets::Paragraph,
        },
        Action, Component, ComponentAccessors,
    },
};

component! {
    // mandatory struct definition
    pub struct Home {
        drank_matetuis: u32
    },

    // optional children definition
    // can also be defined for example in a custom ::new() method using the children! macro
    children => {
        "fps-counter" => fps::FpsComponent::new().as_active() // by default, the component is not
                                                              // active. An inactive component will
                                                              // not receive any events
    }
}

impl Home {
    // custom methods can be defined here
    fn layout(&self, area: Rect) -> [Rect; 2] {
        Layout::horizontal([50, 50].iter().map(|&c| Constraint::Percentage(c))).areas(area)
    }
}

impl Component for Home {
    // implement the Component trait for the Home component
    fn receive_message(&mut self, message: String) {
        // react on drink-mate message received from the app
        match message.as_str() {
            "app:drink-mate" => {
                self.drank_matetuis += 1;
                // if I drank too much mate, we shound stop, too much mate, mate!

                if self.drank_matetuis > 10 {
                    self.send_action(Action::Quit);
                    // we also have the
                    // self.send("message") method to send a custom message to the event-bus
                    // then other components can react to this message
                }
            }
            _ => {}
        }

        // when we override the receive_message method, the default behavior is not executed.
        // by default, the message is passed to all children components. If we want to keep this
        // behavior, we need to call the pass_message_to_children method.
        // there are convenience methods to "reimplement" the default behavior for all
        // Component methods that can be overridden:

        // pass_message_to_children(self, message);
    }

    fn draw(&mut self, f: &mut matetui::Frame<'_>, area: ratatui::prelude::Rect) {
        let [left, right] = self.layout(area);
        let lines = vec![
            Line::from("Hi!"),
            Line::from("Press <d> to drink a mate and <q> to quit."),
            Line::from(""),
            Line::from(format!("You drank {} mates! 🧉", self.drank_matetuis)),
        ];

        let p = Paragraph::new(lines);

        // get the fps child component and draw it
        let fps = self.child_mut("fps-counter").unwrap();
        fps.draw(f, right);

        f.render_widget(p, left);
    }
}
