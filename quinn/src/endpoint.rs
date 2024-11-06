use std::{future::Future, net::SocketAddr, sync::{Arc, Mutex}};
use bevy_ecs::prelude::*;
use bevy_tasks::{IoTaskPool, Task};
use crate::config::{ClientVerification, ServerAuthentication};

#[derive(Component)]
pub struct Endpoint {
    handle: EndpointHandle,
}

impl Endpoint {
    pub fn new(
        socket: SocketAddr,
        auth: ServerAuthentication,
        verify: ClientVerification,
    ) -> Endpoint {
        let handle = Arc::new(());

        let ptr = EndpointRef(Arc::new(EndpointInner {
            handle: handle.clone(),
            state: Mutex::new(EndpointState {
                quinn: todo!(),
            }),
        }));

        let task = EndpointTask::new(EndpointTaskConfig {
            task_pool: IoTaskPool::get(),
            ptr: ptr.clone(),
        });

        return Endpoint {
            handle: EndpointHandle {
                handle,
                task,
                ptr,
            }
        }
    }
}

struct EndpointHandle {
    handle: Arc<()>,
    task: EndpointTask,
    ptr: EndpointRef,
}

/// Endpoint data.
struct EndpointInner {
    handle: Arc<()>,

    state: Mutex<EndpointState>,
}

/// Mutable endpoint state.
struct EndpointState {
    quinn: quinn_proto::Endpoint,
}

/// A clonable, shared reference to an [`EndpointInner`].
#[derive(Clone)]
struct EndpointRef(Arc<EndpointInner>);

/// Config used to build an [`EndpointTask`].
struct EndpointTaskConfig<'a> {
    task_pool: &'a IoTaskPool,
    ptr: EndpointRef
}

/// Handle to endpoint logic running asynchronously.
struct EndpointTask(Task<()>);

impl EndpointTask {
    fn new(config: EndpointTaskConfig) -> Self {
        let task = async move {

        };

        return Self(config.task_pool.spawn(task));
    }
}