#![windows_subsystem = "windows"]

mod app;
mod cloudflared;
mod config;
mod error;
mod rdp;
mod tunnel;
mod ui;

use app::App;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(App::new, App::update, App::view)
        .title("Rust RDP")
        .subscription(App::subscription)
        .theme(App::theme)
        .centered()
        .run()
}
