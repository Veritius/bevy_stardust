use bevy_ecs::system::Resource;

/// The runtime for async processing.
#[derive(Resource)]
pub struct Runtime {
    runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn new(
        threads: usize
    ) -> Runtime {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name("network")
            .worker_threads(threads)
            .enable_all()
            .build()
            .unwrap();

        return Self::from_tokio(runtime);
    }

    pub fn from_tokio(
        runtime: tokio::runtime::Runtime
    ) -> Runtime {
        return Runtime {
            runtime,
        }
    }

    pub(crate) fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }
}