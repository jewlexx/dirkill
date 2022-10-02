use std::thread;

use app::{App, FILES};
use clap::Parser;
use crossterm::terminal::disable_raw_mode;
use dirlib::args::DirKillArgs;

mod app;

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

    let qualified_dir = dunce::canonicalize(&args.dir)?;

    let mut app = App::new();

    let files_thread = thread::spawn(move || {
        *FILES.lock() = Some(dirlib::get_files(&args, qualified_dir));
    });

    app.run()?;

    files_thread.join().unwrap();

    Ok(())
}
