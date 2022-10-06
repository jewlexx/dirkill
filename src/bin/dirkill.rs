use std::thread;

use clap::Parser;
use dirlib::{
    app::{App, FILES},
    args::DirKillArgs,
};

#[macro_use]
extern crate tracing;

fn main() -> anyhow::Result<()> {
    dirlib::init_tracing();
    std::panic::set_hook(Box::new(|_| dirlib::app::pre_exit().unwrap()));

    info!("Starting dirkill");

    let args = DirKillArgs::parse();

    let qualified_dir = dunce::canonicalize(&args.dir)?;

    let mut app = App::new();

    let files_thread = thread::spawn(move || {
        let files = dirlib::get_files(&args, qualified_dir);
        // println!("{:?}", files);
        *FILES.lock() = Some(files);
    });

    app.run()?;

    files_thread.join().unwrap();

    Ok(())
}
