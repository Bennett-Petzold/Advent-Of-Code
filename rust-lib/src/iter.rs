/// An iterator from an array is both exactly sized and double ended
pub trait ArrayIter<T>: ExactSizeIterator<Item = T> + DoubleEndedIterator<Item = T> {}

impl<I, T> ArrayIter<T> for I where I: ExactSizeIterator<Item = T> + DoubleEndedIterator<Item = T> {}

// -------------------------------------------------- //

pub struct ToExactIter<I> {
    len: usize,
    inner: I,
}

impl<I> ToExactIter<I> {
    /// `len` must be the number of elements remaining in `inner`.
    pub fn new(inner: I, len: usize) -> Self {
        Self { inner, len }
    }
}

impl<I> ExactSizeIterator for ToExactIter<I>
where
    I: Iterator,
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<I> DoubleEndedIterator for ToExactIter<I>
where
    I: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.len = self.len.saturating_sub(1);
        self.inner.next_back()
    }
}

impl<I> Iterator for ToExactIter<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.len = self.len.saturating_sub(1);
        self.inner.next()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.len = self.len.saturating_sub(n + 1);
        self.inner.nth(n)
    }
}
