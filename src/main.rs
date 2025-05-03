#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]
#![allow(clippy::missing_errors_doc)]

use app::App;
use args::Args;
use clap::Parser;
use comms::Comms;
use ratatui::style::Color;
use tokio::runtime::Handle;

mod app;
mod args;
mod color;
mod files;

pub mod comms;
#[cfg(debug_assertions)]
mod logs;
mod sorting;

#[macro_use]
extern crate tracing;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Do not bother initializing tracing if we are not in debug mode
    #[cfg(debug_assertions)]
    assert!(logs::init_tracing().is_ok(), "Failed to initialize tracing");

    std::panic::set_hook(Box::new(|info| {
        app::pre_exit().unwrap();

        eprintln!("{info}");
    }));

    info!("Starting dirkill");

    let args = Args::parse();

    let color = match args.color.as_ref().map(color::parse_hex) {
        Some(Ok(color)) => color,
        _ => Color::Yellow,
    };

    let comms = Comms::default();

    let app = App::new(color, comms.clone());

    let sorting_task = app.sort_entries();

    let discovery_task = tokio::spawn(async move {
        args.get_files(
            dunce::canonicalize(&args.dir).expect("Failed to canonicalize path"),
            &comms,
        )
        .await;
    });

    if app.run().await.is_err() {
        error!("Failed to run app");
    }
    let handle = Handle::current();
    let metrics = handle.metrics();
    dbg!(metrics.num_alive_tasks());

    discovery_task.abort();
    discovery_task.await.unwrap();
    sorting_task.abort();
    _ = sorting_task.await;

    let metrics = handle.metrics();
    dbg!(metrics.num_alive_tasks());
}
