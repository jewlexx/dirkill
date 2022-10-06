use std::{
    fs::File,
    io::Write,
    ops::Range,
    path::{Path, PathBuf},
};

use args::DirKillArgs;
use num_traits::Num;
use tracing::Level;
use tracing_subscriber::fmt::{format::FmtSpan, MakeWriter};

use crate::app::ENTRIES;

pub mod app;
pub mod args;

#[macro_use]
extern crate tracing;

pub trait IntWrapType<T: std::cmp::PartialOrd<T>>:
    Num
    + Copy
    + Clone
    + std::ops::AddAssign<usize>
    + std::ops::SubAssign<usize>
    + std::ops::Add<usize>
    + std::ops::Sub<usize>
    + std::cmp::PartialOrd<T>
{
}

impl<T: std::cmp::PartialOrd> IntWrapType<T> for usize where usize: std::cmp::PartialOrd<T> {}

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
        self.file.write(buf)
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

#[derive(Default)]
pub struct IntWrap<T: IntWrapType<T>>(T, Range<T>);

impl<T: IntWrapType<T>> IntWrap<T> {
    pub const fn new(value: T, range: Range<T>) -> Self {
        Self(value, range)
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn increase(&mut self, increase: usize) -> &T {
        self.0 += increase;

        if !self.1.contains(&self.0) {
            self.0 = self.1.start;
        }

        &self.0
    }

    pub fn decrease(&mut self, decrease: usize) -> &T {
        self.0 -= decrease;

        if !self.1.contains(&self.0) {
            self.0 = self.1.end;
        }

        &self.0
    }

    pub fn bump(&mut self) -> &T {
        self.0 += 1;

        if !self.1.contains(&self.0) {
            self.0 = self.1.start;
        }

        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub size: u64,
    pub entry: walkdir::DirEntry,
}

impl From<walkdir::DirEntry> for DirEntry {
    fn from(entry: walkdir::DirEntry) -> Self {
        let meta = entry.metadata().unwrap();

        let size = if meta.is_dir() {
            fs_extra::dir::get_size(entry.path()).unwrap()
        } else {
            meta.len()
        };

        Self { size, entry }
    }
}

#[tracing::instrument]
pub fn get_files(args: &DirKillArgs, search_dir: impl AsRef<Path> + core::fmt::Debug) {
    let search_dir = search_dir.as_ref();
    let target_dir = &args.target;

    debug!("Searching for files in {:?}", search_dir);

    let mut iter = walkdir::WalkDir::new(search_dir)
        .follow_links(false)
        .into_iter();

    debug!("Getting files");

    loop {
        match iter.next() {
            Some(Ok(entry)) => {
                let entry: DirEntry = entry.into();
                let path = entry.entry.path();
                let is_target = path
                    .components()
                    .last()
                    .map(|c| c.as_os_str() == target_dir)
                    .unwrap_or(false);

                if is_target && entry.entry.file_type().is_dir() {
                    debug!("Found dir {}", path.display());
                    ENTRIES.lock().push(entry);
                }
            }
            None => break,
            _ => break,
        }
    }

    *crate::app::LOADING.lock() = false;
}
