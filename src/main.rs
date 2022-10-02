use std::thread;

use app::App;
use clap::Parser;
use crossterm::terminal::disable_raw_mode;

mod app;

#[derive(Debug, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
struct DirKillArgs {
    #[clap(
        short,
        long,
        default_value = "node_modules",
        help = "The directory to remove"
    )]
    target: String,
}

fn pre_exit() -> anyhow::Result<()> {
    use crossterm::{execute, terminal::LeaveAlternateScreen};
    use std::io;

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|_| pre_exit().unwrap()));

    let args = DirKillArgs::parse();

    let mut app = App::new();

    app.run()?;

    Ok(())
}
