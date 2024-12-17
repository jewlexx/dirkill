use std::path::{Path, PathBuf};

use clap::Parser;
use crossbeam_channel::Sender;

use crate::UpdateChannel;

#[derive(Debug, Clone, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
pub struct Args {
    #[clap(
        short,
        long,
        default_value = "node_modules",
        help = "The directory to remove"
    )]
    pub target: PathBuf,

    #[clap(short, long, default_value = ".", help = "The directory to search")]
    pub dir: PathBuf,

    #[clap(
        long,
        help = "The highlight color to use for the selected entry. Must be a hex value"
    )]
    pub color: Option<String>,

    #[clap(short = 'l', long, help = "Whether or not to follow symlinks")]
    pub follow_links: bool,
}

impl Args {
    pub fn get_files(
        &self,
        search_dir: impl AsRef<Path> + core::fmt::Debug,
        tx: &Sender<UpdateChannel>,
    ) {
        let search_dir = search_dir.as_ref();
        let target_dir = &self.target;

        debug!("Searching for files in {:?}", search_dir);

        let mut iter = walkdir::WalkDir::new(search_dir)
            .follow_links(self.follow_links)
            .into_iter();

        debug!("Getting files");

        loop {
            match iter.next() {
                Some(Ok(entry)) => {
                    // debug!("Found entry {}", entry.path().display());
                    let path = entry.path();
                    let is_target = path
                        .components()
                        .last()
                        .is_some_and(|x| x.as_os_str() == target_dir);

                    if is_target && entry.file_type().is_dir() {
                        // Do not continue searching the directory, as it is the target directory
                        iter.skip_current_dir();
                        tx.send(Some(entry.into())).unwrap();
                    }
                    // assert!(!ENTRIES.is_locked());
                    // assert!(!CHANGED.is_locked());
                }
                None => break,
                _ => {}
            }

            // if ENTRIES.is_locked() {
            //     trace!("Entries lock is locked");
            // }
            // if CHANGED.is_locked() {
            //     trace!("Changed lock is locked");
            // }
        }

        *crate::app::LOADING.lock() = false;
        assert!(!crate::app::LOADING.is_locked());
    }
}
