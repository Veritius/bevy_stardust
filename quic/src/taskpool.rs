use std::{collections::VecDeque, future::Future, ops::{AddAssign, SubAssign}, sync::{atomic::{AtomicUsize, Ordering}, Condvar, Mutex, OnceLock}};
use async_task::{Runnable, Task};

pub(crate) static NETWORK_TASK_POOL: OnceLock<NetworkTaskPool> = OnceLock::new();

pub(crate) struct NetworkTaskPool {
    queue: Mutex<VecDeque<IncompleteTask>>,
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
        task_pool.queue.lock().unwrap().push_back(IncompleteTask { runnable });
        task_pool.cvar.notify_one();
    }

    fn init() -> Self {
        NetworkTaskPool {
            queue: Mutex::new(VecDeque::new()),
            cvar: Condvar::new(),
        }
    }
}

struct IncompleteTask {
    runnable: Runnable,
}

pub(crate) fn get_task_pool() -> &'static NetworkTaskPool {
    NETWORK_TASK_POOL.get_or_init(NetworkTaskPool::init)
}

static THREAD_MANAGER: ThreadManager = ThreadManager {
    tasks: AtomicUsize::new(0),
};

struct ThreadManager {
    tasks: AtomicUsize,
}

pub(crate) struct ThreadPoints(usize);

impl ThreadPoints {
    pub fn new(points: usize) -> Self {
        THREAD_MANAGER.tasks.fetch_add(points, Ordering::Relaxed);
        return ThreadPoints(points);
    }
}

impl Drop for ThreadPoints {
    fn drop(&mut self) {
        THREAD_MANAGER.tasks.fetch_sub(self.0, Ordering::Relaxed);
    }
}

impl AddAssign<usize> for ThreadPoints {
    fn add_assign(&mut self, rhs: usize) {
        THREAD_MANAGER.tasks.fetch_add(rhs, Ordering::Relaxed);
        self.0 += rhs;
    }
}

impl SubAssign<usize> for ThreadPoints {
    fn sub_assign(&mut self, rhs: usize) {
        THREAD_MANAGER.tasks.fetch_sub(rhs, Ordering::Relaxed);
        self.0 -= rhs;
    }
}