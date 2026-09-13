use core::fmt;
use core::num::NonZeroUsize;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NonMaxUsize(NonZeroUsize);
impl NonMaxUsize {
    pub const fn get(&self) -> usize {
        !self.0.get()
    }
    pub const fn new(val: usize) -> Option<Self> {
        match NonZeroUsize::new(!val) {
            Some(inner) => Some(Self(inner)),
            None => None,
        }
    }
    /// SAFETY: `val` must not be `usize::MAX`
    pub const unsafe fn new_unchecked(val: usize) -> Self {
        // SAFETY: `val` is not `usize::MAX` so `!val` is not 0
        Self(unsafe { NonZeroUsize::new_unchecked(!val) })
    }
}

impl fmt::Debug for NonMaxUsize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NonMaxUsize").field(&self.get()).finish()
    }
}
