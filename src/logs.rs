use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use tracing::Level;
use tracing_subscriber::fmt::{format::FmtSpan, MakeWriter};

struct TracingWriter {
    file: File,
}

impl TracingWriter {
    pub fn new(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut path = file_path.as_ref().to_owned();
        let file_name = chrono::Local::now()
            .format("dir-kill.%Y-%m-%d_%H-%M-%S.log")
            .to_string();

        path.push(file_name);

        let file = File::create(path)?;

        Ok(Self { file })
    }
}

impl Write for TracingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let ascii_chars: Vec<u8> = buf.iter().filter(|c| c.is_ascii()).cloned().collect();

        self.file.write(&ascii_chars)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

struct TracingWriterWrapper;

impl MakeWriter<'_> for TracingWriterWrapper {
    type Writer = TracingWriter;

    fn make_writer(&self) -> Self::Writer {
        TracingWriter::new(get_log_path()).unwrap()
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
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE | FmtSpan::ENTER | FmtSpan::EXIT)
            .with_thread_names(true)
            .with_max_level(Level::DEBUG)
            .with_writer(TracingWriterWrapper)
            .init();
    }

    Ok(())
}
