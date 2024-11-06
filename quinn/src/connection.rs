use std::sync::{Arc, Mutex};
use async_task::Task;
use bevy_ecs::prelude::*;
use crate::{config::{ClientAuthentication, ServerVerification}, runtime::Runtime};
use super::endpoint::Endpoint;

#[derive(Component)]
pub struct Connection {
    handle: ConnectionHandle,
}

impl Connection {
    pub fn new(
        runtime: impl Runtime,
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
struct ConnectionTaskConfig {
    ptr: ConnectionRef
}

/// Handle to connection logic running asynchronously.
struct ConnectionTask(Task<()>);

impl ConnectionTask {
    fn new(
        runtime: impl Runtime,
        config: ConnectionTaskConfig,
    ) -> Self {
        let task = async move {

        };

        return Self(runtime.spawn(task));
    }
}