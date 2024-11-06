use std::future::Future;

use async_task::Task;

/// A runtime for data.
pub trait Runtime {
    fn spawn<T>(&self, task: impl Future<Output = T> + Send + 'static) -> Task<T>;
}