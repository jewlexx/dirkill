use std::path::Path;

use crate::{app::ENTRIES, args::DirKillArgs};

pub fn recursive_size(path: impl AsRef<Path>) -> u64 {
    let path = path.as_ref();
    let mut size = 0;

    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            size += recursive_size(&entry.path());
        }
    } else {
        size += path.metadata().unwrap().len();
    }

    size
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub size: u64,
    pub entry: walkdir::DirEntry,
    /// None if the entry hasn't been touched. Some(true) if the entry has been deleted, and Some(false) if it is being deleted
    pub deleting: Option<bool>,
}

impl From<walkdir::DirEntry> for DirEntry {
    fn from(entry: walkdir::DirEntry) -> Self {
        let size = recursive_size(entry.path());

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
        .follow_links(args.follow_links)
        .into_iter();

    debug!("Getting files");

    loop {
        match iter.next() {
            Some(Ok(entry)) => {
                debug!("Found entry {}", entry.path().display());
                let entry: DirEntry = entry.into();
                let path = entry.entry.path();
                let is_target = path
                    .components()
                    .last()
                    .map(|x| x.as_os_str() == target_dir)
                    .unwrap_or(false);

                if is_target && entry.entry.file_type().is_dir() {
                    // Do not continue searching the directory, as it is the target directory
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

#[cfg(test)]
mod tests {
    use super::recursive_size;

    #[test]
    fn test_recursive_size() {
        let src_size = recursive_size("src");
        let extra_src_size = fs_extra::dir::get_size("src").unwrap();

        assert_eq!(src_size, extra_src_size);
    }
}
