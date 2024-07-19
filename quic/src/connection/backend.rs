/// A connection associated with a [`Backend`](crate::backend::Backend) implementation.
pub trait ConnectionBackend
where
    Self: Send + Sync,
{
    /// Returns `true` if the connection is fully closed and drained,
    /// and that dropping it is guaranteed to not cause data loss.
    fn is_closed(&self) -> bool;
}