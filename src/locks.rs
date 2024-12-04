//! Map a Mutex Lock
//!
//! Can theoretically be anything but designed primarily for Mutex Locks

#[allow(clippy::module_name_repetitions)]
/// Map a mutex lock
pub trait LockMap<'a, 'g, T: ?Sized, G: 'g> {
    /// Maps a mutex lock of T to a value of U
    fn map<F, U>(&'a self, f: F) -> U
    where
        F: FnOnce(G) -> U + 'g,
        'a: 'g;
}

impl<'a, 'g, T> LockMap<'a, 'g, T, parking_lot::MutexGuard<'g, T>> for parking_lot::Mutex<T> {
    fn map<F, U>(&'a self, f: F) -> U
    where
        F: FnOnce(parking_lot::MutexGuard<'g, T>) -> U,
        'a: 'g,
    {
        f(self.lock())
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::Mutex;
    use quork::prelude::*;

    use super::LockMap;

    static MUTEX: Mutex<bool> = Mutex::new(false);

    #[test]
    fn test_lock_map() {
        MUTEX.map(|mut m| m.flip());
    }
}
