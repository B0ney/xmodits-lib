pub trait BitFlag {
    /// Returns ``true`` if all the **``1s``** in ``self`` exists the ``1s`` on the rhs
    fn is_set(self, rhs: impl Into<u8>) -> bool
    where
        Self: Sized + Copy + Into<u8>,
    {
        (self.into() & rhs.into()) == self.into()
    }
}

impl BitFlag for u8 {}