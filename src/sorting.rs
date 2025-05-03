pub mod column;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use quork::prelude::FlipImmut;
use tokio::sync::Mutex;

use crate::{app::Entry, comms::Comms, files::DirEntry};

pub use column::Column;

#[derive(Debug, Default, Clone)]
pub struct Sorting {
    column: Column,
    inverted: Arc<AtomicBool>,
    sorted: Arc<Mutex<Vec<Entry>>>,
    comms: Comms,
}

impl Sorting {
    pub fn invert(&mut self) {
        let inverted = self.inverted.load(Ordering::Relaxed);
        self.inverted.store(!inverted, Ordering::Relaxed);
    }

    pub fn inverted(&self) -> bool {
        self.inverted.load(Ordering::Relaxed)
    }

    #[tracing::instrument(skip(self), ret)]
    pub fn column(&self) -> Column {
        self.column.clone()
    }

    #[tracing::instrument(skip(self))]
    pub fn set_column(&mut self, column: Column) {
        self.column = column.clone();
        assert_eq!(self.column(), column);
    }

    pub fn switch_column(&mut self) {
        self.column.flip();
    }

    pub async fn add_entry(&self, new_entry: &DirEntry) {
        self.sorted.lock().await.push(Entry::from(new_entry));
    }

    #[tracing::instrument(skip(self))]
    pub async fn sort(&self) {
        let mut unsorted = self.sorted.lock().await;
        unsorted.sort_unstable_by(|a, b| {
            match self.column() {
                column if column.is_name() => a.original.entry.path().cmp(b.original.entry.path()),
                // Sorting is inverse here, because we want the larger size to be first
                column if column.is_size() => b.size.cmp(&a.size),
                _ => unreachable!(),
            }
        });

        if self.inverted() {
            unsorted.reverse();
        }

        self.comms.set_changed(true);
        self.comms.set_loading(false);
    }

    pub async fn sorted(&self) -> Vec<Entry> {
        self.sorted.lock().await.clone()
    }

    pub fn blocking_sorted(&self) -> Vec<Entry> {
        thread::scope(|scope| {
            scope
                .spawn(|| self.sorted.blocking_lock().clone())
                .join()
                .unwrap()
        })
    }

    pub fn sorted_mut(&self) -> Arc<Mutex<Vec<Entry>>> {
        self.sorted.clone()
    }
}

// #[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
// pub enum Column {
//     #[default]
//     Name,
//     Size,
// }
