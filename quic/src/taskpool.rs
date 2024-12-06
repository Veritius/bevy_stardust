use std::{future::Future, sync::OnceLock};
use async_task::{Runnable, Task};
use crossbeam_deque::Injector;

pub(crate) static NETWORK_TASK_POOL: OnceLock<NetworkTaskPool> = OnceLock::new();

pub(crate) struct NetworkTaskPool {
    global_queue: Injector<IncompleteTask>,
}

impl NetworkTaskPool {
    pub(crate) fn spawn<F, O>(&self, future: F) -> Task<O>
    where
        F: Future<Output = O>,
        F: Send + Sync + 'static,
        O: Send + 'static,
    {
        let (runnable, task) = async_task::spawn(future, Self::schedule);
        self.global_queue.push(IncompleteTask { runnable });
        return task;
    }

    fn schedule(runnable: Runnable) {
        get_task_pool().global_queue.push(IncompleteTask { runnable })
    }

    fn init() -> Self {
        NetworkTaskPool {
            global_queue: Injector::new(),
        }
    }
}

struct IncompleteTask {
    runnable: Runnable,
}

pub(crate) fn get_task_pool() -> &'static NetworkTaskPool {
    NETWORK_TASK_POOL.get_or_init(NetworkTaskPool::init)
}