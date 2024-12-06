use std::{collections::VecDeque, future::Future, sync::{Condvar, Mutex, OnceLock}};
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