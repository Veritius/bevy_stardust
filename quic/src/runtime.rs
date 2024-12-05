use std::{future::Future, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle};
use async_task::{Runnable, Task};
use bevy_ecs::system::Resource;
use crossbeam_deque::Injector;

/// Builder for a [`Runtime`].
pub struct RuntimeBuilder {
    threads: usize,
}

impl RuntimeBuilder {
    /// Creates a new [`RuntimeBuilder`].
    pub fn new() -> RuntimeBuilder {
        RuntimeBuilder {
            threads: 1,
        }
    }

    /// Sets the number of worker threads used.
    /// Defaults to `1`.
    pub fn threads(mut self, count: usize) -> Self {
        self.threads = count;
        return self;
    }

    /// Builds the [`Runtime`].
    pub fn build(self) -> Runtime {
        let mut runtime = Runtime {
            workers: Vec::with_capacity(self.threads),
            state: Arc::new(State {
                tasks: Injector::new(),
            }),
        };

        runtime.add_workers(self.threads);

        return runtime;
    }
}

/// An owned handle to threads handling network traffic.
/// 
/// Can be used as a Bevy [`Resource`].
pub struct Runtime {
    workers: Vec<Worker>,
    state: Arc<State>,
}

impl Runtime {
    /// Returns the number of active worker threads.
    pub fn thread_count(&self) -> usize {
        self.workers.len()
    }

    /// Adds additional worker threads.
    pub fn add_workers(&mut self, amount: usize) {
        if amount == 0 { return } // do nothing

        let iter = (0..amount)
            .map(|_| Worker::new(
                format!("quic worker"),
                self.state.clone(),
            ));

        self.workers.extend(iter);
    }

    /// Shuts down worker threads.
    pub fn remove_workers(&mut self, amount: usize) {
        todo!()
    }

    /// Returns the corresponding [`Handle`] for this [`Runtime`].
    pub fn handle(&self) -> Handle {
        Handle {
            state: self.state.clone(),
        }
    }
}

impl Resource for Runtime {}

/// A reference to the internal state of a [`Runtime`].
#[derive(Clone)]
pub struct Handle {
    state: Arc<State>,
}

impl Handle {
    pub(crate) fn spawn<F, O>(&self, fut: F) -> Task<O>
    where
        F: Future<Output = O>,
        F: Send + Sync + 'static,
        O: Send + 'static,
    {
        let state = self.state.clone();
        let schedule = move |runnable| state.tasks.push(runnable);
        let (runnable, task) = async_task::spawn(fut, schedule);
        runnable.schedule();
        return task;
    }
}

/// A worker thread.
struct Worker {
    thread: JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
}

impl Worker {
    fn new(
        name: impl Into<String>,
        state: Arc<State>,
    ) -> Worker {
        // Notification system for shutting down threads
        let shutdown = Arc::new(AtomicBool::new(false));

        // Start a new worker thread
        let thread_shutdown = shutdown.clone();
        let thread = std::thread::Builder::new()
            .name(name.into())
            .spawn(move || Worker::task(
                state,
                thread_shutdown,
            ))
            .unwrap();

        // We're done here
        return Worker {
            thread,
            shutdown,
        };
    }

    fn task(
        state: Arc<State>,
        shutdown: Arc<AtomicBool>,
    ) {
        // Thread-local queue of tasks needing completion
        let local = crossbeam_deque::Worker::new_lifo();

        loop {
            // Check if we've been signalled to shut down
            if shutdown.load(Ordering::Relaxed) {
                return; // Stop processing immediately
            }

            if local.is_empty() {
                // Fill up our local queue from the runtime's global queue
                let _ = state.tasks.steal_batch_with_limit(&local, 8);
            }

            // Run any tasks we have in our local queue
            if let Some(runnable) = local.pop() {
                runnable.run();
            }
        }
    }
}

/// Internal state for execution.
struct State {
    tasks: Injector<Runnable>,
}