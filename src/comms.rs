use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam_channel::{Receiver, RecvError, SendError, Sender};

use crate::files::DirEntry;

pub type UpdateChannel = Option<DirEntry>;

#[derive(Debug, Clone)]
pub struct Comms {
    entries_tx: Arc<Sender<UpdateChannel>>,
    entries_rx: Arc<Receiver<UpdateChannel>>,

    changed: Arc<AtomicBool>,
    loading: Arc<AtomicBool>,
}

impl Default for Comms {
    fn default() -> Self {
        let (entries_tx, entries_rx) = crossbeam_channel::unbounded::<UpdateChannel>();

        Self {
            entries_tx: Arc::new(entries_tx),
            entries_rx: Arc::new(entries_rx),

            changed: Arc::new(AtomicBool::new(false)),
            loading: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Comms {
    pub fn set_loading(&self, loading: bool) {
        self.loading.store(loading, Ordering::Relaxed);
    }
    #[must_use]
    pub fn loading(&self) -> bool {
        self.loading.load(Ordering::Relaxed)
    }

    pub fn set_changed(&self, changed: bool) {
        self.changed.store(changed, Ordering::Relaxed);
    }
    #[must_use]
    pub fn changed(&self) -> bool {
        self.changed.load(Ordering::Relaxed)
    }

    pub fn push_entry(&self, entry: DirEntry) -> Result<(), SendError<()>> {
        self.entries_tx.send(Some(entry)).map_err(|_| SendError(()))
    }

    pub fn push_sort_tick(&self) -> Result<(), SendError<()>> {
        self.entries_tx.send(None).map_err(|_| SendError(()))
    }

    pub fn pop_entry(&self) -> Result<Option<DirEntry>, RecvError> {
        self.entries_rx.recv()
    }
}
