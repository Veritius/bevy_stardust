use std::{future::Future, sync::Arc, thread::JoinHandle};
use async_executor::Executor;
use async_task::Task;
use futures_lite::{future::block_on, FutureExt};

/// The runtime for async events.
pub struct Runtime {
    executor: RuntimeExecutor,
    threads: Box<[JoinHandle<()>]>,
    shutdown: crossbeam_channel::Sender<()>,
}

impl Runtime {
    pub fn new(
        threads: usize,
    ) -> Runtime {
        let thread_count = threads;

        let executor = Arc::new(Executor::new());
        let mut threads = Vec::with_capacity(threads);
        let (sh_tx, sh_rx) = crossbeam_channel::unbounded();

        for _ in 0..thread_count {
            let executor = executor.clone();
            let shutdown: crossbeam_channel::Receiver<()> = sh_rx.clone();

            threads.push(std::thread::spawn(move || {
                let ticker = async {
                    loop { executor.clone().tick().await }
                };

                let shutdown = async {
                    shutdown.recv()
                };

                block_on(executor.clone().run(ticker.or(shutdown))).unwrap();
            }));
        }

        return Runtime {
            executor: RuntimeExecutor {
                inner: executor,
            },

            threads: threads.into(),
            shutdown: sh_tx,
        };
    }

    pub(crate) fn spawn<O: Send + 'static>(
        &self,
        task: impl Future<Output = O>,
    ) -> Task<O> {
        todo!()
    }

    pub(crate) fn executor(&self) -> RuntimeExecutor {
        self.executor.clone()
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeExecutor {
    inner: Arc<Executor<'static>>,
}

impl RuntimeExecutor {
    pub(crate) fn spawn<O: Send + 'static>(
        &self,
        task: impl Future<Output = O>,
    ) -> Task<O> {
        todo!()
    }
}