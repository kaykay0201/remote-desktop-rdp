#![windows_subsystem = "windows"]

mod app;
mod capture;
mod config;
mod error;
mod input_handler;
mod network;
mod process;
mod protocol;
mod tailscale;
mod ui;
mod updater;

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
