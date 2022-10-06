use std::thread;

use clap::Parser;
use dirlib::{
    app::{App, FILES},
    args::DirKillArgs,
    DirEntry,
};
use parking_lot::Mutex;

static ENTRIES: Mutex<Vec<DirEntry>> = Mutex::new(Vec::new());

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
        dirlib::get_files(&args, qualified_dir, &ENTRIES);
        // println!("{:?}", files);
        *FILES.lock() = Some(files);
    });

    app.run()?;

    Ok(())
}
