use std::{future::Future, sync::Arc, thread::JoinHandle};
use async_executor::Executor;
use async_task::Task;

/// A runtime for asynchronous tasks.
pub trait Runtime {
    fn spawn<O>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O>;
}

impl<T: Runtime> Runtime for &T {
    #[inline]
    fn spawn<O>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O> {
        (*self).spawn(task)
    }
}

/// A very minimal runtime.
pub struct MinimalRuntime {
    executors: Arc<Executor<'static>>,
    threads: Vec<JoinHandle<()>>,
}

impl Runtime for MinimalRuntime {
    fn spawn<O>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O> {
        todo!()
    }
}