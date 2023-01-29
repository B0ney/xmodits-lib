pub trait BitFlag: Sized + Copy + Into<u8> {
    /// Returns ``true`` if all the **``1s``** in ``rhs`` exist in ``lhs`` (``self``)
    fn is_set(self, rhs: impl Into<u8> + Copy) -> bool {
        (self.into() & rhs.into()) == rhs.into()
    }
}

impl<T: Sized + Copy + Into<u8>> BitFlag for T {}
