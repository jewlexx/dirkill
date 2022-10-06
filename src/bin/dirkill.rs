use std::thread;

use clap::Parser;
use dirlib::{app::App, args::DirKillArgs};

#[macro_use]
extern crate tracing;

fn main() -> anyhow::Result<()> {
    dirlib::init_tracing();
    std::panic::set_hook(Box::new(|_| dirlib::app::pre_exit().unwrap()));

    info!("Starting dirkill");

    let args = DirKillArgs::parse();

    let qualified_dir = dunce::canonicalize(&args.dir)?;

    let mut app = App::new();

    thread::spawn(move || {
        dirlib::get_files(&args, qualified_dir);
    });
    // .join()
    // .unwrap();

    app.run()?;

    Ok(())
}
