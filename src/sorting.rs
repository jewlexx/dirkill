use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use parking_lot::Mutex;

use crate::{app::Entry, comms::Comms, files::DirEntry};

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

    pub fn column(&self) -> Column {
        self.column
    }

    pub fn set_column(&mut self, column: Column) {
        self.column = column;
    }

    pub fn switch_column(&mut self) {
        self.set_column(match self.column {
            Column::Name => Column::Size,
            Column::Size => Column::Name,
        });
    }

    pub fn add_entry(&self, new_entry: &DirEntry) {
        self.sorted.lock().push(Entry::from(new_entry));
    }

    pub fn sort(&self) {
        self.comms.set_loading(true);
        let column = self.column();
        let mut unsorted = self.sorted.lock().clone();
        unsorted.sort_unstable_by(|a, b| {
            match column {
                Column::Name => a.original.entry.path().cmp(b.original.entry.path()),
                // Sorting is inverse here, because we want the larger size to be first
                Column::Size => b.size.cmp(&a.size),
            }
        });

        if self.inverted() {
            unsorted.reverse();
        }

        self.comms.set_changed(true);
        self.comms.set_loading(false);
    }

    pub fn sorted(&self) -> Arc<Mutex<Vec<Entry>>> {
        self.sorted.clone()
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Column {
    #[default]
    Name,
    Size,
}
