use std::path::Path;

use crate::{app::ENTRIES, args::DirKillArgs};

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub size: u64,
    pub entry: walkdir::DirEntry,
    /// None if the entry hasn't been touched. Some(true) if the entry has been deleted, and Some(false) if it is being deleted
    pub deleting: Option<bool>,
}

impl From<walkdir::DirEntry> for DirEntry {
    fn from(entry: walkdir::DirEntry) -> Self {
        let meta = entry.metadata().unwrap();

        let size = if meta.is_dir() {
            fs_extra::dir::get_size(entry.path()).unwrap()
        } else {
            meta.len()
        };

        Self {
            size,
            entry,
            deleting: None,
        }
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
                debug!("Found entry {}", entry.path().display());
                let entry: DirEntry = entry.into();
                let path = entry.entry.path();
                let is_target = {
                    let is_based = path
                        .components()
                        .filter(|c| c.as_os_str() == target_dir)
                        .count()
                        == 1;

                    let is_target = path
                        .components()
                        .last()
                        .map(|x| x.as_os_str() == target_dir)
                        .unwrap_or(false);

                    is_based && is_target
                };

                if is_target && entry.entry.file_type().is_dir() {
                    iter.skip_current_dir();
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
