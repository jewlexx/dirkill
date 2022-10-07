#[macro_use]
extern crate tracing;

use std::thread;

use clap::Parser;

use app::App;
use args::DirKillArgs;

mod app;
mod args;
mod color;
mod files;
mod logs;

fn main() -> anyhow::Result<()> {
    let guard = logs::init_tracing()?;
    std::panic::set_hook(Box::new(|_| {
        app::pre_exit().unwrap();
    }));

    info!("Starting dirkill");

    let args = DirKillArgs::parse();

    let qualified_dir = dunce::canonicalize(&args.dir)?;

    let color = args.color.parse::<u32>()?;

    let rgb = match colors_transform::Rgb::from_hex_str(&args.color) {
        Ok(v) => v,
        Err(_) => {
            error!("Invalid color provided");
            std::process::exit(1);
        }
    };

    let mut app = App::new(rgb);

    thread::spawn(move || {
        files::get_files(&args, qualified_dir);
    });

    app.run()?;

    drop(guard);

    Ok(())
}
