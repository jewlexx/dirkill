use std::ops::Range;

use num_traits::Num;

pub trait IntWrapType<T: std::cmp::PartialOrd<T>>:
    Num
    + Clone
    + std::ops::AddAssign<usize>
    + std::ops::SubAssign<usize>
    + std::ops::Add<usize>
    + std::ops::Sub<usize>
    + std::cmp::PartialOrd<T>
{
}

impl<T: std::cmp::PartialOrd> IntWrapType<T> for usize where usize: std::cmp::PartialOrd<T> {}

pub struct IntWrap<T: IntWrapType<T>>(T, Range<T>);

impl Default for IntWrap<usize> {
    fn default() -> Self {
        Self(0, 0..100)
    }
}

impl<T: IntWrapType<T>> IntWrap<T> {
    pub fn new(value: T, range: Range<T>) -> Self {
        Self(value, range)
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn increase(&mut self, increase: usize) -> &T {
        self.0 += increase;

        if self.1.contains(&self.0) {
            &self.0
        } else {
            self.0 -= increase;

            &self.1.end
        }
    }

    pub fn decrease(&mut self, decrease: usize) -> &T {
        self.0 -= decrease;

        if self.1.contains(&self.0) {
            &self.0
        } else {
            self.0 += decrease;

            &self.1.start
        }
    }

    pub fn bump(&mut self) -> &T {
        self.0 += 1;

        if self.1.contains(&self.0) {
            &self.0
        } else {
            self.0 -= 1;

            &self.1.end
        }
    }
}
