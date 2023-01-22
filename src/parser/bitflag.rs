pub trait BitFlag {
    /// Returns ``true`` if all the **``1s``** in ``rhs`` exist in ``lhs`` (``self``)
    fn is_set(self, rhs: impl Into<u8> + Copy) -> bool
    where
        Self: Sized + Copy + Into<u8>,
    {
        (self.into() & rhs.into()) == rhs.into()
    }
}

impl<T: Sized + Copy + Into<u8>> BitFlag for T {}
