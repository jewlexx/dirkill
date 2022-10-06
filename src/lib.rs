use std::{ops::Range, path::Path};

use args::DirKillArgs;
use num_traits::Num;

use crate::app::ENTRIES;

pub mod app;
pub mod args;
pub mod logs;

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
            _ => {}
        }
    }

    *crate::app::LOADING.lock() = false;
}
