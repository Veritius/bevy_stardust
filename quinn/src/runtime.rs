use std::{future::Future, sync::Arc, thread::JoinHandle};
use async_executor::Executor;
use async_task::Task;
use futures_lite::{future::block_on, FutureExt};

/// A runtime for asynchronous tasks.
pub trait Runtime {
    fn spawn<O: Send + 'static>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O>;
}

impl<T: Runtime> Runtime for &T {
    #[inline]
    fn spawn<O: Send + 'static>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O> {
        (*self).spawn(task)
    }
}

/// A very minimal runtime.
pub struct MinimalRuntime {
    executor: Arc<Executor<'static>>,
    threads: Box<[JoinHandle<()>]>,
    shutdown: async_channel::Sender<()>,
}

impl Runtime for MinimalRuntime {
    fn spawn<O: Send + 'static>(&self, task: impl Future<Output = O> + Send + 'static) -> Task<O> {
        self.executor.spawn(task)
    }
}

impl MinimalRuntime {
    pub fn new(
        threads: usize,
    ) -> MinimalRuntime {
        let thread_count = threads;

        let executor = Arc::new(Executor::new());
        let mut threads = Vec::with_capacity(threads);
        let (sh_tx, sh_rx) = async_channel::unbounded::<()>();

        for _ in 0..thread_count {
            let executor = executor.clone();
            let shutdown = sh_rx.clone();

            threads.push(std::thread::spawn(move || {
                let ticker = async {
                    loop { executor.clone().tick().await }
                };

                block_on(executor.clone().run(ticker.or(shutdown.recv()))).unwrap();
            }));
        }

        return MinimalRuntime {
            executor,
            threads: threads.into(),
            shutdown: sh_tx,
        };
    }
}