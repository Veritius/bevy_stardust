use std::sync::{Arc, Mutex};
use async_task::Task;
use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use crate::{config::{ClientAuthentication, ServerVerification}, endpoint::EndpointEvents, runtime::Runtime};
use super::endpoint::Endpoint;

#[derive(Component)]
pub struct Connection {
    handle: ConnectionHandle,
}

impl Connection {
    pub fn new(
        runtime: &Runtime,
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
    events: EndpointEvents,
}

/// A clonable, shared reference to an [`ConnectionInner`].
#[derive(Clone)]
pub(crate) struct ConnectionRef(Arc<ConnectionInner>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ConnectionId(usize);

#[derive(Default)]
pub(crate) struct ConnectionIdSource(usize);

impl ConnectionIdSource {
    pub fn increment(&mut self) -> ConnectionId {
        let idx = self.0;
        self.0 += 1;
        return ConnectionId(idx);
    }
}

pub(crate) struct ConnectionEvents {
    pub quinn_rx: Receiver<quinn_proto::ConnectionEvent>,
    pub quinn_tx: Sender<quinn_proto::EndpointEvent>,
}

/// Config used to build an [`ConnectionTaask`].
struct ConnectionTaskConfig {
    ptr: ConnectionRef
}

/// Handle to connection logic running asynchronously.
struct ConnectionTask(Task<()>);

impl ConnectionTask {
    fn new(
        runtime: &Runtime,
        config: ConnectionTaskConfig,
    ) -> Self {
        let task = async move {

        };

        return Self(runtime.spawn(task));
    }
}