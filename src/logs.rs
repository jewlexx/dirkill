use std::{fs::File, path::PathBuf};

use strip_ansi_escapes::Writer;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;

fn get_log_path() -> PathBuf {
    let base_path = std::env::current_dir().unwrap().join("logs");

    if !base_path.exists() {
        std::fs::create_dir(&base_path).unwrap();
    } else {
        let read = std::fs::read_dir(&base_path).unwrap();

        if read.count() > 10 {
            std::fs::remove_dir_all(&base_path).unwrap();
            std::fs::create_dir(&base_path).unwrap();
        }
    }

    base_path
}

pub fn init_tracing() -> anyhow::Result<Option<WorkerGuard>> {
    #[cfg(not(profile = "release"))]
    {
        let mut path = get_log_path();
        let file_name = chrono::Local::now()
            .format("dir-kill.%Y-%m-%d_%H-%M-%S.log")
            .to_string();

        path.push(file_name);

        let file = File::create(path).unwrap();

        let (non_blocking, guard) = tracing_appender::non_blocking(Writer::new(file));

        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE | FmtSpan::ENTER | FmtSpan::EXIT)
            .with_thread_names(true)
            .with_max_level(Level::DEBUG)
            .with_writer(non_blocking)
            .init();

        Ok(Some(guard))
    }

    #[cfg(profile = "release")]
    Ok(None)
}
