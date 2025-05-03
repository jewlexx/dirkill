use std::{
    convert::Infallible,
    sync::{atomic::AtomicBool, Arc},
};

use quork::{prelude::Flip, traits::flip::FlipImmut};

#[derive(Debug, Clone)]
pub struct Column(Arc<AtomicBool>);

impl Column {
    const NAME_BOOL: bool = false;
    const SIZE_BOOL: bool = true;

    pub fn new(state: bool) -> Self {
        Self(Arc::new(AtomicBool::new(state)))
    }

    pub fn name() -> Self {
        Self::new(Self::NAME_BOOL)
    }

    pub fn size() -> Self {
        Self::new(Self::SIZE_BOOL)
    }

    pub fn is_name(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_size(&self) -> bool {
        !self.is_name()
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::name()
    }
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.is_name() == other.is_name()
    }
}

impl Eq for Column {}

impl Flip for Column {
    fn flipped(&self) -> Self {
        if self.is_name() {
            Self::size()
        } else {
            Self::name()
        }
    }
}

impl<'a> FlipImmut<'a, Column> for Column {
    type Error = Infallible;

    fn try_flip(&'a self) -> Result<(), Self::Error> {
        _ = self.0.try_flip();
        Ok(())
    }

    fn try_flipped(&'a self) -> Result<Column, Self::Error> {
        Ok(Flip::flipped(self))
    }
}
