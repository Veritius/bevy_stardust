use std::{future::Future, num::NonZero, sync::{atomic::{AtomicUsize, Ordering}, Condvar, Mutex, OnceLock}, thread::{self, available_parallelism}, time::Duration};
use async_task::{Runnable, Task};
use concurrent_queue::ConcurrentQueue;

const DESPAWN_TIMEOUT: Duration = Duration::from_millis(500);

pub(crate) static NETWORK_TASK_POOL: OnceLock<NetworkTaskPool> = OnceLock::new();

pub(crate) struct NetworkTaskPool {
    queue: ConcurrentQueue<IncompleteTask>,
    cvar: Condvar,

    thread_index: AtomicUsize,
    idle_threads: AtomicUsize,
    spawn_state: Mutex<ThreadSpawnState>,
}

impl NetworkTaskPool {
    pub(crate) fn spawn<F, O>(&self, future: F) -> Task<O>
    where
        F: Future<Output = O>,
        F: Send + Sync + 'static,
        O: Send + 'static,
    {
        let (runnable, task) = async_task::spawn(future, Self::schedule);
        Self::schedule(runnable);
        return task;
    }

    fn init() -> Self {
        NetworkTaskPool {
            queue: ConcurrentQueue::unbounded(),
            cvar: Condvar::new(),

            thread_index: AtomicUsize::new(0),
            idle_threads: AtomicUsize::new(0),

            spawn_state: Mutex::new(ThreadSpawnState {
                total_threads: 0,
                max_threads: None,
            }),
        }
    }

    fn schedule(runnable: Runnable) {
        let task_pool = get_task_pool();
        let _ = task_pool.queue.push(IncompleteTask { runnable });
        task_pool.cvar.notify_one();
        task_pool.grow();
    }

    fn grow(&'static self) {
        todo!()
    }
}

struct IncompleteTask {
    runnable: Runnable,
}

struct ThreadSpawnState {
    total_threads: usize,
    max_threads: Option<NonZero<usize>>,
}

pub(crate) fn get_task_pool() -> &'static NetworkTaskPool {
    NETWORK_TASK_POOL.get_or_init(NetworkTaskPool::init)
}

fn start_worker_thread(
    task_pool: &'static NetworkTaskPool,
    state: &mut ThreadSpawnState,
) {
    // Default value for when a max thread count can't be found.
    const UNRETRIEVABLE_LIMIT_DEFAULT: NonZero<usize> = NonZero::new(usize::MAX).unwrap();

    // Calculate the maximum number of threads we can create
    let max_threads = available_parallelism()
        .unwrap_or(UNRETRIEVABLE_LIMIT_DEFAULT)
        .max(state.max_threads.unwrap_or(UNRETRIEVABLE_LIMIT_DEFAULT));

    // Check that spawning a new thread won't exceed the limit
    if state.total_threads >= max_threads.into() { return }

    // Try to spawn the thread
    let res = thread::Builder::new()
        .name(format!("quic-{}", task_pool.thread_index.fetch_add(1, Ordering::Relaxed)))
        .spawn(move || worker_thread(task_pool));

    match res {
        Ok(_) => {
            // We've successfully added the thread, so we can increment the counter
            state.total_threads += 1;
            task_pool.idle_threads.fetch_add(1, Ordering::Relaxed);
        },

        Err(err) => {
            // It's likely that we've hit some kind of imposed system limit, so update the max thread count accordingly.
            // This can occur when available_parallelism() returns an inaccurate estimate, so it's worth accounting for.
            state.max_threads = Some(NonZero::new(state.total_threads).unwrap());

            log::error!("Failed to spawn new thread for processing. This can happen i: {err}");
        },
    }
}

fn worker_thread(
    task_pool: &'static NetworkTaskPool,
) {
    loop {

    }
}