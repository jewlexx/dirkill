use std::thread;

use parking_lot::Mutex;

use app::App;
use clap::Parser;
use crossterm::terminal::disable_raw_mode;

mod app;

static APP: Mutex<App> = Mutex::new(App::new());

#[derive(Debug, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
struct DirKillArgs {}

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

    let ui_thread = thread::spawn(|| {
        APP.lock().run()?;

        anyhow::Ok(())
    });

    thread::spawn(|| {
        let app = APP.lock();

        println!("Able to lock app");
    })
    .join()
    .unwrap();

    ui_thread.join().unwrap()?;

    Ok(())
}
