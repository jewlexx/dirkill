#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

use std::thread;

use app::App;
use args::Args;
use clap::Parser;
use ratatui::style::Color;

mod app;
mod args;
mod color;
mod files;

#[cfg(debug_assertions)]
mod logs;

#[macro_use]
extern crate tracing;

fn main() {
    // Do not bother initializing tracing if we are not in debug mode
    #[cfg(debug_assertions)]
    assert!(logs::init_tracing().is_ok(), "Failed to initialize tracing");

    std::panic::set_hook(Box::new(|info| {
        app::pre_exit().unwrap();

        eprintln!("{info}");
    }));

    info!("Starting dirkill");

    let args = Args::parse();

    let color = match args.color.as_ref().map(color::parse_hex) {
        Some(Ok(color)) => color,
        _ => Color::Yellow,
    };

    let mut app = App::new(color);

    thread::spawn(move || {
        args.get_files(dunce::canonicalize(&args.dir).expect("Failed to canonicalize path"));
    });

    if app.run().is_err() {
        error!("Failed to run app");
    };
}
