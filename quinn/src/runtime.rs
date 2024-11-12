use std::{future::Future, sync::Arc, thread::JoinHandle};
use async_executor::Executor;
use async_task::Task;
use futures_lite::{future::block_on, FutureExt};

/// The runtime for async events.
pub struct Runtime {
    executor: Arc<Executor<'static>>,
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
                    loop { executor.tick().await }
                };

                let shutdown = async {
                    shutdown.recv()
                };

                block_on(executor.clone().run(ticker.or(shutdown))).unwrap();
            }));
        }

        return Runtime {
            executor,
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
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self.shutdown.send(());
    }
}