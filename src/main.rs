use std::thread;

use clap::Parser;

use app::App;
use args::DirKillArgs;
use tui::style::Color;

mod app;
mod args;
mod color;
mod files;
mod logs;

#[macro_use]
extern crate tracing;

fn main() {
    if logs::init_tracing().is_err() && cfg!(not(profile = "release")) {
        panic!("Failed to initialize tracing");
    };

    std::panic::set_hook(Box::new(|info| {
        println!("{}", info);
        app::pre_exit().unwrap();
    }));

    info!("Starting dirkill");

    let args = DirKillArgs::parse();

    let qualified_dir = dunce::canonicalize(&args.dir).expect("Failed to canonicalize path");

    let color = match args.color.as_ref().map(color::parse_hex) {
        Some(Ok(color)) => color,
        _ => Color::Yellow,
    };

    let mut app = App::new(color);

    thread::spawn(move || {
        files::get_files(&args, qualified_dir);
    });

    if app.run().is_err() {
        error!("Failed to run app");
    };
}
