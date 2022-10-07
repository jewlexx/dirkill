#[macro_use]
extern crate tracing;

use std::thread;

use clap::Parser;

use app::App;
use args::DirKillArgs;

pub mod app;
pub mod args;
pub mod files;
pub mod logs;

fn main() -> anyhow::Result<()> {
    let guard = logs::init_tracing()?;
    std::panic::set_hook(Box::new(|_| {
        app::pre_exit().unwrap();
    }));

    info!("Starting dirkill");

    let args = DirKillArgs::parse();

    let qualified_dir = dunce::canonicalize(&args.dir)?;

    let mut app = App::new();

    thread::spawn(move || {
        files::get_files(&args, qualified_dir);
    });

    app.run()?;

    drop(guard);

    Ok(())
}
