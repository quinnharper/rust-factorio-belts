pub(crate) trait OptionPolyfill<T> {
    fn err_or<U>(self, ok: U) -> Result<U, T>;
    fn err_or_else<U, F>(self, ok: F) -> Result<U, T>
    where
        F: FnOnce() -> U;
}
impl<T> OptionPolyfill<T> for Option<T> {
    fn err_or<U>(self, ok: U) -> Result<U, T> {
        match self {
            Some(err) => Err(err),
            None => Ok(ok),
        }
    }

    fn err_or_else<U, F>(self, ok: F) -> Result<U, T>
    where
        F: FnOnce() -> U,
    {
        match self {
            Some(err) => Err(err),
            None => Ok(ok()),
        }
    }
}
