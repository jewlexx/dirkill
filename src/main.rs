use std::{ffi::OsString, thread};

use clap::Parser;

use app::App;
use ratatui::style::Color;

mod app;
mod color;
mod files;

#[cfg(debug_assertions)]
mod logs;

#[macro_use]
extern crate tracing;

#[derive(Debug, Clone, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
pub struct Args {
    #[clap(
        short,
        long,
        default_value = "node_modules",
        help = "The directory to remove"
    )]
    pub target: OsString,

    #[clap(short, long, default_value = ".", help = "The directory to search")]
    pub dir: OsString,

    #[clap(
        long,
        help = "The highlight color to use for the selected entry. Must be a hex value"
    )]
    pub color: Option<String>,

    #[clap(short = 'l', long, help = "Whether or not to follow symlinks")]
    pub follow_links: bool,
}

fn main() {
    // Do not bother initializing tracing if we are not in debug mode
    #[cfg(debug_assertions)]
    if logs::init_tracing().is_err() {
        panic!("Failed to initialize tracing");
    };

    std::panic::set_hook(Box::new(|info| {
        app::pre_exit().unwrap();

        eprintln!("{info}");
    }));

    info!("Starting dirkill");

    let args = Args::parse();

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
