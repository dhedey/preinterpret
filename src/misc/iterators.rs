pub(crate) enum EitherIterator<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Iterator for EitherIterator<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIterator::Left(l) => l.next(),
            EitherIterator::Right(r) => r.next(),
        }
    }
}
