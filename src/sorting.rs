use crate::{app::Entry, comms::Comms, files::DirEntry};

#[derive(Debug, Default, Clone)]
pub struct Sorting {
    column: Column,
    inverted: bool,
    sorted: Vec<Entry>,
    comms: Comms,
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

    pub fn add_entry(&mut self, new_entry: &DirEntry) {
        self.sorted.push(Entry::from(new_entry));
    }

    pub fn sort(&mut self) {
        let column = self.column();
        self.sorted.sort_unstable_by(|a, b| {
            match column {
                Column::Name => a.original.entry.path().cmp(b.original.entry.path()),
                // Sorting is inverse here, because we want the larger size to be first
                Column::Size => b.size.cmp(&a.size),
            }
        });

        if self.inverted() {
            self.sorted.reverse();
        }

        self.comms.set_changed(true);
    }

    pub fn sorted(&self) -> &[Entry] {
        &self.sorted
    }

    pub fn sorted_mut(&mut self) -> &mut Vec<Entry> {
        &mut self.sorted
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Column {
    #[default]
    Name,
    Size,
}
