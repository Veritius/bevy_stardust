use std::{collections::VecDeque, future::Future, sync::{Arc, Mutex}, thread::JoinHandle};
use async_task::{Runnable, Task};

/// A runtime for asynchronous tasks.
pub trait Runtime {
    fn spawn<O>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O>;
}

impl<T> Runtime for Arc<T>
where
    T: Runtime,
{
    fn spawn<O>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O> {
        self.as_ref().spawn(task)
    }
}

/// A very minimal runtime.
pub struct MinimalRuntime {
    queue: Mutex<VecDeque<Runnable>>,
    threads: Mutex<Vec<JoinHandle<()>>>,
}

impl Runtime for MinimalRuntime {
    fn spawn<O>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O> {
        todo!()
    }
}