use std::{cmp::Ordering, future::Future, sync::{Condvar, Mutex, OnceLock}, thread};
use async_task::{Runnable, Task};
use concurrent_queue::ConcurrentQueue;

pub(crate) static NETWORK_TASK_POOL: OnceLock<NetworkTaskPool> = OnceLock::new();

pub(crate) struct NetworkTaskPool {
    queue: ConcurrentQueue<IncompleteTask>,
    cvar: Condvar,
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

    fn schedule(runnable: Runnable) {
        let task_pool = get_task_pool();
        let _ = task_pool.queue.push(IncompleteTask { runnable });
        task_pool.cvar.notify_one();
    }
}

struct IncompleteTask {
    runnable: Runnable,
}

pub(crate) fn get_task_pool() -> &'static NetworkTaskPool {
    NETWORK_TASK_POOL.get_or_init(|| {
        return NetworkTaskPool {
            queue: ConcurrentQueue::unbounded(),
            cvar: Condvar::new(),
        };
    })
}

static WORKER_THREAD_STATE: Mutex<WorkerThreadsState> = Mutex::new(WorkerThreadsState {
    current: 0,
    desired: 0,
    index: 0,
});

struct WorkerThreadsState {
    current: usize,
    desired: usize,
    index: usize,
}

/// Object for controlling the number of threads handling QUIC networking.
pub enum WorkerThreads {}

impl WorkerThreads {
    /// Sets the number of threads to `count`.
    pub fn set(count: usize) {
        let mut lock = WORKER_THREAD_STATE.lock().unwrap();
        lock.desired = count;

        match lock.current.cmp(&lock.desired) {
            Ordering::Less => Self::increase_threads_to_fit(&mut lock),
            Ordering::Greater => get_task_pool().cvar.notify_all(),
            Ordering::Equal => { /* do nothing */},
        }
    }

    /// Returns the target number of threads.
    pub fn desired() -> usize {
        let lock = WORKER_THREAD_STATE.lock().unwrap();
        return lock.current;
    }

    /// Returns the current number of threads.
    /// 
    /// Does not include threads yet to spawn, or in the process of spawning.
    pub fn current() -> usize {
        let lock = WORKER_THREAD_STATE.lock().unwrap();
        return lock.current;
    }

    fn increase_threads_to_fit(
        state: &mut WorkerThreadsState,
    ) {
        // If we're at the desired count, don't do anything.
        if state.desired <= state.current { return }

        // Get task pool once so we can give it to the worker thread fn
        // This also prevents us from repeatedly checking the OnceLock
        let task_pool = get_task_pool();
    
        // Spawn enough threads to make up the difference.
        // Subtract won't cause problems as we just compared them.
        for _ in 0..(state.desired - state.current) {
            let res = thread::Builder::new()
                .name(format!("quic-{}", state.index))
                .spawn(|| worker_thread(task_pool));
    
            if let Err(err) = res {
                log::error!("Error while spawning threads to match desired amount: {err}");
                return;
            }
            
            state.index += 1;
        }
    }
}

fn worker_thread(
    task_pool: &'static NetworkTaskPool,
) {
    loop {

    }
}