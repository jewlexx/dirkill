use clap::Parser;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod app;

#[derive(Debug, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
struct DirKillArgs {}

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|_| {
        disable_raw_mode().unwrap();
    }));

    let args = DirKillArgs::parse();

    let mut app = app::App::new();

    enable_raw_mode()?;

    app.run()?;

    disable_raw_mode()?;

    Ok(())
}
