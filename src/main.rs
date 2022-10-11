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

#[cfg(not(profile = "release"))]
#[macro_use]
extern crate tracing;

#[cfg(profile = "release")]
#[macro_use]
mod log_macros {
    #[macro_export]
    macro_rules! info {
        ($($arg:tt)*) => {};
    }

    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)*) => {};
    }

    #[macro_export]
    macro_rules! error {
        ($($arg:tt)*) => {};
    }

    #[macro_export]
    macro_rules! warn {
        ($($arg:tt)*) => {};
    }

    #[macro_export]
    macro_rules! trace {
        ($($arg:tt)*) => {};
    }
}

fn main() -> anyhow::Result<()> {
    let guard = logs::init_tracing()?;
    std::panic::set_hook(Box::new(|_| {
        app::pre_exit().unwrap();
    }));

    info!("Starting dirkill");

    let args = DirKillArgs::parse();

    let qualified_dir = dunce::canonicalize(&args.dir)?;

    let color = if let Some(ref hex) = args.color {
        color::parse_hex(hex)?
    } else {
        Color::Yellow
    };

    let mut app = App::new(color);

    thread::spawn(move || {
        files::get_files(&args, qualified_dir);
    });

    app.run()?;

    drop(guard);

    Ok(())
}
