use std::{
    fs::File,
    path::{Path, PathBuf},
};

use strip_ansi_escapes::Writer;
use tracing::Level;
use tracing_subscriber::fmt::{format::FmtSpan, MakeWriter};

struct TracingWriter {
    file_path: PathBuf,
}

impl TracingWriter {
    pub fn new(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut path = file_path.as_ref().to_owned();
        let file_name = chrono::Local::now()
            .format("dir-kill.%Y-%m-%d_%H-%M-%S.log")
            .to_string();

        path.push(file_name);

        Ok(Self { file_path: path })
    }
}

impl MakeWriter<'_> for TracingWriter {
    type Writer = Writer<File>;

    fn make_writer(&self) -> Self::Writer {
        let file = File::create(&self.file_path).unwrap();

        Writer::new(file)
    }
}

fn get_log_path() -> PathBuf {
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            let base_path = std::env::current_dir().unwrap().join("logs");

            if !base_path.exists() {
                std::fs::create_dir(&base_path).unwrap();
            }

            base_path
        } else {
            std::env::temp_dir()
        }
    }
}

pub fn init_tracing() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        let (non_blocking, _guard) =
            tracing_appender::non_blocking(TracingWriter::new(get_log_path())?.make_writer());

        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE | FmtSpan::ENTER | FmtSpan::EXIT)
            .with_thread_names(true)
            .with_max_level(Level::DEBUG)
            .with_writer(non_blocking)
            .init();
    }

    Ok(())
}
