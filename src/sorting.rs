use crate::app::{Entry, CHANGED, ENTRIES};

#[derive(Debug, Default, Clone)]
pub struct Sorting {
    column: Column,
    inverted: bool,
    sorted: Vec<Entry>,
}

impl Sorting {
    pub fn invert(&mut self) {
        self.inverted = !self.inverted;
    }

    pub fn inverted(&self) -> bool {
        self.inverted
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

    pub fn sort(&mut self) {
        let mut unsorted_entries = ENTRIES.lock();

        unsorted_entries.sort_unstable_by(|a, b| match self.column() {
            Column::Name => a.entry.path().cmp(b.entry.path()),
            // Sorting is inverse here, because we want the larger size to be first
            Column::Size => b.size.cmp(&a.size),
        });

        if self.inverted() {
            unsorted_entries.reverse();
        }

        self.sorted = unsorted_entries.iter().map(Entry::from).collect();
        *CHANGED.lock() = true;
    }

    pub fn sorted(&self) -> &[Entry] {
        &self.sorted
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Column {
    #[default]
    Name,
    Size,
}
