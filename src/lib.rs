use std::{ops::Range, path::Path};

use args::DirKillArgs;
use num_traits::Num;
use tracing_subscriber::fmt::format::FmtSpan;

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

pub fn init_tracing() {
    let (writer, _guard) = tracing_appender::non_blocking(std::io::stdout());

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE | FmtSpan::ENTER | FmtSpan::EXIT)
        .with_writer(writer)
        .with_thread_names(true)
        .init();
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

pub fn get_files(args: &DirKillArgs, search_dir: impl AsRef<Path>) -> Vec<DirEntry> {
    let search_dir = search_dir.as_ref();
    let target_dir = &args.target;

    let iter = walkdir::WalkDir::new(search_dir)
        .follow_links(false)
        .into_iter();

    let mut entries: Vec<DirEntry> = iter
        .filter_map(|entry| entry.ok())
        .map(|entry| -> DirEntry { entry.into() })
        .filter(|entry| entry.entry.path().ends_with(target_dir))
        .collect();

    entries.sort_by(|a, b| a.size.cmp(&b.size));

    entries
}
