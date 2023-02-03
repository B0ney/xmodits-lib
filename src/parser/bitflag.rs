pub trait BitFlag: Sized + Copy + Into<u8> {
    /// Returns ``true`` if all of the **``1s``** in ``other`` exist in (``self``)
    fn contains(self, other: impl Into<u8> + Copy) -> bool {
        (self.into() & other.into()) == other.into()
    }

    // / Returns ``true`` if all the **``1s``** in ``lhs`` (``self``) exist in ``rhs``
    // /
    // /// TODO: come up with a clearer, less confusing name
    // fn is_set_for_left(self, rhs: impl Into<u8> + Copy) -> bool {
    //     (rhs.into() & self.into() ) == self.into()
    // }
}

impl<T: Sized + Copy + Into<u8>> BitFlag for T {}
