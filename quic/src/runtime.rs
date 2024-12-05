use std::future::Future;

/// A handle to an asynchronous runtime for I/O.
pub trait Runtime {
    /// Spawn a future to be run asynchronously in the background.
    fn spawn(&self, future: dyn Future<Output = ()>);
}