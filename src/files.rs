use std::path::Path;

// #[tracing::instrument]
pub fn recursive_size(path: impl AsRef<Path> + std::fmt::Debug) -> u64 {
    let path = path.as_ref();
    let mut size = 0;

    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            size += recursive_size(entry.path());
        }
    } else {
        size += path.metadata().map(|x| x.len()).unwrap_or_default();
    }

    size
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum DeletionState {
    #[default]
    Ready,
    Deleting,
    Deleted,
    Error,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub size: u64,
    pub entry: walkdir::DirEntry,
    /// None if the entry hasn't been touched. Some(true) if the entry has been deleted, and Some(false) if it is being deleted
    pub deletion_state: DeletionState,
}

impl From<walkdir::DirEntry> for DirEntry {
    fn from(entry: walkdir::DirEntry) -> Self {
        let size = recursive_size(entry.path());

        Self {
            size,
            entry,
            deletion_state: DeletionState::default(),
        }
    }
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
