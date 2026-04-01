use std::collections::VecDeque;

// A type with known length and first element (unless it's empty) that
// can be repeatedly iterated over.
pub trait Sequence {
    type Item;

    fn len(&self) -> usize;
    fn front(&self) -> Option<Self::Item>;
    fn iter(&self) -> impl Iterator<Item = Self::Item>;
}

// A `Sequence` based on a `VecDeque` where the (enumerated) elements
// are modified by a function.
pub struct MappedVecDeque<'data, F> {
    pub inner: &'data VecDeque<(f64, f64)>,
    pub f: F,
}

impl<'a> Sequence for &'a VecDeque<(f64, f64)> {
    type Item = &'a (f64, f64);

    fn len(&self) -> usize {
        VecDeque::len(self)
    }

    fn front(&self) -> Option<Self::Item> {
        VecDeque::front(self)
    }

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        VecDeque::iter(self)
    }
}

impl<'data, F> Sequence for MappedVecDeque<'data, F>
where
    F: Fn((usize, &(f64, f64))) -> (f64, f64),
{
    type Item = (f64, f64);

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn front(&self) -> Option<Self::Item> {
        self.inner.front().map(|p| (self.f)((0, p)))
    }

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.inner.iter().enumerate().map(&self.f)
    }
}
