pub trait BitFlag: Sized + Copy + Into<u8> {
    /// Returns ``true`` if all of the **``1s``** in ``other`` exist in (``self``)
    fn contains(self, other: impl Into<u8> + Copy) -> bool {
        (self.into() & other.into()) == other.into()
    }
}

impl<T: Sized + Copy + Into<u8>> BitFlag for T {}
