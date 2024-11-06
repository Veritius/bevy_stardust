use std::sync::{Arc, Mutex};
use bevy_ecs::prelude::*;
use bevy_tasks::{IoTaskPool, Task};
use crate::config::{ClientAuthentication, ServerVerification};
use super::endpoint::Endpoint;

#[derive(Component)]
pub struct Connection {
    handle: ConnectionHandle,
}

impl Connection {
    pub fn new(
        endpoint: &mut Endpoint,
        auth: ClientAuthentication,
        verify: ServerVerification,
    ) -> Self {
        todo!()
    }
}
struct ConnectionHandle {
    handle: Arc<()>,
    task: ConnectionTask,
    ptr: ConnectionRef,
}

/// Connection data.
struct ConnectionInner {
    handle: Arc<()>,

    state: Mutex<ConnectionState>,
}

/// Mutable connection state.
struct ConnectionState {
    quinn: quinn_proto::Connection,
}

/// A clonable, shared reference to an [`ConnectionInner`].
#[derive(Clone)]
struct ConnectionRef(Arc<ConnectionInner>);

/// Config used to build an [`ConnectionTaask`].
struct ConnectionTaskConfig<'a> {
    task_pool: &'a IoTaskPool,
    ptr: ConnectionRef
}

/// Handle to connection logic running asynchronously.
struct ConnectionTask(Task<()>);

impl ConnectionTask {
    fn new(config: ConnectionTaskConfig) -> Self {
        let task = async move {

        };

        return Self(config.task_pool.spawn(task));
    }
}