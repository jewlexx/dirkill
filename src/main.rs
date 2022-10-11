use std::thread;

use clap::Parser;

use app::App;
use args::DirKillArgs;
use tui::style::Color;

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
    if logs::init_tracing().is_err() {
        panic!("Failed to initialize tracing");
    };

    std::panic::set_hook(Box::new(|info| {
        app::pre_exit().unwrap();

        println!("{}", info);
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
