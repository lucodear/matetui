mod fps;
mod home;
use {
    home::Home,
    matetui::{components, kb, Action, App, ComponentAccessors},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // the home component is the root component of the app
    let home = Home::default().as_active();

    let mut app = App::default()
        .with_components(components![home])
        .with_frame_rate(24)
        .with_tick_rate(24)
        .with_keybindings(kb![
            "<ctrl-c>" => Action::Quit, // quit the app when pressing ctrl-c
            "<q>" => Action::Quit,      // quit the app when pressing q
            "<d>" => "app:drink-mate"   // send custom message when pressing d (all components will
                                        // receive this message and act accordingly)
        ]);

    // run the app
    app.run().await?;

    println!("Goodbye!");
    Ok(())
}
