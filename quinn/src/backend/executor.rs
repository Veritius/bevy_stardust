use std::{sync::Arc, thread::{self, JoinHandle}};
use async_executor::Executor;
use bevy_ecs::prelude::*;
use crate::config::BackendConfig;

#[derive(Resource)]
pub(crate) struct BackendExecutor {
    executor: Arc<Executor<'static>>,
    threads: Box<[JoinHandle<()>]>,
}

impl BackendExecutor {
    pub fn init(
        config: BackendConfig,
    ) -> Self {
        // Create a new executor instance
        let executor = Arc::new(Executor::new());

        // Create all the threads for running things
        let threads = (0..config.threads)
            .map(|_| {
                let executor = executor.clone();
                thread::spawn(move || {
                    loop { executor.try_tick(); }
                })
            })
            .collect();

        // We're done
        return Self {
            executor,
            threads,
        };
    }

    #[inline]
    pub fn executor(&self) -> &Executor<'static> {
        &self.executor
    }

    #[inline]
    pub fn executor_arc(&self) -> Arc<Executor<'static>> {
        self.executor.clone()
    }
}