/// Sorts an iterator if a condition is met.
/// This avoids collecting the iterator
/// if it shouldn't be sorted.
// Overenginereed? Yes. Had fun implementing? Also yes.
pub fn sort_iter_if<T, I, F>(iter: I, sort: bool, compare: F) -> impl Iterator<Item = T>
where
    I: Iterator<Item = T>,
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    if sort {
        let mut items: Vec<_> = iter.collect();
        items.sort_by(compare);
        TwoIter::I(items.into_iter())
    } else {
        TwoIter::J(iter)
    }
}

enum TwoIter<I, J> {
    I(I),
    J(J),
}
impl<T, I, J> Iterator for TwoIter<I, J>
where
    I: Iterator<Item = T>,
    J: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TwoIter::I(i) => i.next(),
            TwoIter::J(j) => j.next(),
        }
    }
}
