use std::{cmp::Ordering, future::Future, sync::{atomic::{self, AtomicBool}, Mutex, OnceLock}, thread, time::Duration};
use async_task::{Runnable, Task};
use concurrent_queue::ConcurrentQueue;
use crossbeam_channel::{Receiver, Sender};

const THREAD_TIMEOUT: Duration = Duration::from_millis(500);

pub(crate) static NETWORK_TASK_POOL: OnceLock<NetworkTaskPool> = OnceLock::new();

pub(crate) struct NetworkTaskPool {
    block_insertions: AtomicBool,
    queue: ConcurrentQueue<IncompleteTask>,

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
        if task_pool.block_insertions.load(atomic::Ordering::Relaxed) { return }
        let _ = task_pool.queue.push(IncompleteTask { runnable });
        task_pool.waker_tx.send(()).unwrap();
    }
}

struct IncompleteTask {
    runnable: Runnable,
}

pub(crate) fn get_task_pool() -> &'static NetworkTaskPool {
    NETWORK_TASK_POOL.get_or_init(|| {
        let (waker_tx, waker_rx) = crossbeam_channel::unbounded();

        return NetworkTaskPool {
            block_insertions: AtomicBool::new(false),
            queue: ConcurrentQueue::unbounded(),

            waker_tx,
            waker_rx,
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
    /// Sets the desired number of threads.
    /// 
    /// Threads will be added/removed to try and match the desired value.
    /// If adding threads fails, an error is returned. Removing threads cannot fail.
    /// 
    /// # Warning
    /// If the number of threads is set to `0`, all tasks are immediately dropped!
    /// All endpoints and connections will be closed immediately, without warning any remote peers.
    pub fn set(count: usize) -> Result<(), std::io::Error> {
        let mut lock = WORKER_THREAD_STATE.lock().unwrap();
        lock.desired = count.into();

        // Block insertions into the task pool if there are no threads to copy them
        // Over time this will slowly but surely drop all the tasks as the threads return
        get_task_pool().block_insertions.store(lock.desired == 0, atomic::Ordering::Relaxed);

        match lock.current.cmp(&lock.desired) {
            Ordering::Less => Self::increase_threads_to_fit(&mut lock)?,
            Ordering::Greater => { /* do nothing, threads automatically wake up every so often */},
            Ordering::Equal => { /* do nothing, we're already at the target number of threads */},
        };

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

fn worker_thread(
    task_pool: &'static NetworkTaskPool,
) {
    loop {
        // Check if we're over budget and stop ourselves if we are, to reduce the number of threads
        let mut lock = WORKER_THREAD_STATE.lock().unwrap();

        if lock.desired == 0 {
            // Drop all tasks that we can and then return.
            // This prevents any tasks from being stuck unprocessed (but still taking up resources)
            // until at least one worker thread exists again, which is not guaranteed within any
            // reasonable amount of time (or at all). Hopefully...
            while let Ok(task) = task_pool.queue.pop() { drop(task); }
        }


        // Check if we're over budget and stop ourselves if we are, to reduce the number of threads.
        if lock.current > lock.desired { lock.current -= 1; return; }

        // Free the lock now rather than later
        // This reduces the chance of blocking other threads
        drop(lock);

        // Consume as many tasks as possible
        while let Ok(task) = task_pool.queue.pop() {
            task.runnable.run();
        }

        // Wait for the next event or time out
        let _ = task_pool.waker_rx.recv_timeout(THREAD_TIMEOUT);
    }
}