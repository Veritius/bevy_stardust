use std::{cmp::Ordering, future::Future, sync::{Mutex, Once, OnceLock}, thread, time::Duration};
use async_task::{Runnable, Task};
use concurrent_queue::ConcurrentQueue;
use crossbeam_channel::{Receiver, Sender};

const THREAD_TIMEOUT: Duration = Duration::from_millis(500);

pub(crate) static NETWORK_TASK_POOL: OnceLock<NetworkTaskPool> = OnceLock::new();

pub(crate) struct NetworkTaskPool {
    fallback_thread: Once,
    task_queue: ConcurrentQueue<Runnable>,

    waker_tx: Sender<()>,
    waker_rx: Receiver<()>,
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

        // Ensures there's at least one thread running to handle network tasks
        task_pool.fallback_thread.call_once(|| {
            thread::Builder::new()
                .name(format!("quic-0"))
                .spawn(|| fallback_thread(task_pool))
                .unwrap(); // shouldn't fail
        });

        let _ = task_pool.task_queue.push(runnable);
        task_pool.waker_tx.send(()).unwrap();
    }
}

pub(crate) fn get_task_pool() -> &'static NetworkTaskPool {
    NETWORK_TASK_POOL.get_or_init(|| {
        let (waker_tx, waker_rx) = crossbeam_channel::unbounded();

        return NetworkTaskPool {
            fallback_thread: Once::new(),
            task_queue: ConcurrentQueue::unbounded(),

            waker_tx,
            waker_rx,
        };
    })
}

static WORKER_THREAD_STATE: Mutex<WorkerThreadsState> = Mutex::new(WorkerThreadsState {
    current: 0,
    desired: 0,
    index: 1, // fallback thread uses 1
});

struct WorkerThreadsState {
    current: usize,
    desired: usize,
    index: usize,
}

/// Object for controlling the number of threads handling QUIC networking.
pub enum WorkerThreads {}

impl WorkerThreads {
    /// Sets the desired number of threads.
    /// 
    /// Threads will be added/removed to try and match the desired value.
    /// If adding threads fails, an error is returned. Removing threads cannot fail.
    /// 
    /// **Note:** There is always one additional thread that cannot be shut down or removed.
    /// This is to make sure that tasks cannot get stuck forever and waste resources.
    pub fn set(value: usize) -> Result<(), std::io::Error> {
        let mut lock = WORKER_THREAD_STATE.lock().unwrap();
        Self::set_inner(&mut lock, value)?;
        return Ok(());
    }

    /// Increases the desired number of threads. See [`set`](Self::set) for more details.
    pub fn increase(value: usize) -> Result<(), std::io::Error> {
        let mut lock = WORKER_THREAD_STATE.lock().unwrap();
        let value = lock.desired + value;
        Self::set_inner(&mut lock, value)?;
        return Ok(());
    }

    /// Decreases the desired number of threads, saturating at zero. See [`set`](Self::set) for more details.
    pub fn decrease(value: usize) -> Result<(), std::io::Error> {
        let mut lock = WORKER_THREAD_STATE.lock().unwrap();
        let value = lock.desired.saturating_sub(value);
        Self::set_inner(&mut lock, value)?;
        return Ok(());
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

    fn set_inner(
        state: &mut WorkerThreadsState,
        value: usize,
    ) -> Result<(), std::io::Error> {
        state.desired = value.into();

        match state.current.cmp(&state.desired) {
            Ordering::Less => Self::increase_threads_to_fit(state)?,
            Ordering::Greater => { /* do nothing, threads automatically wake up every so often */ },
            Ordering::Equal => { /* do nothing, we're already at the target number of threads */ },
        };

        return Ok(())
    }

    fn increase_threads_to_fit(
        state: &mut WorkerThreadsState,
    ) -> Result<(), std::io::Error> {
        // If we're at the desired count, don't do anything.
        if state.desired <= state.current { return Ok(()); }

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
                return Err(err);
            }

            state.index += 1;
        }

        // done
        return Ok(());
    }
}

fn fallback_thread(
    task_pool: &'static NetworkTaskPool,
) {
    loop {
        // Consume as many tasks as possible
        while let Ok(task) = task_pool.task_queue.pop() {
            task.run();
        }

        // Wait for the next event
        let _ = task_pool.waker_rx.recv();
    }
}

fn worker_thread(
    task_pool: &'static NetworkTaskPool,
) {
    loop {
        // Check if we're over budget and stop ourselves if we are, to reduce the number of threads
        let mut lock = WORKER_THREAD_STATE.lock().unwrap();
        if lock.current > lock.desired { lock.current -= 1; return; }

        // Free the lock now rather than later
        // This reduces the chance of blocking other threads
        drop(lock);

        // Consume as many tasks as possible
        while let Ok(task) = task_pool.task_queue.pop() {
            task.run();
        }

        // Wait for the next event or time out
        let _ = task_pool.waker_rx.recv_timeout(THREAD_TIMEOUT);
    }
}