use crate::backend::QuicBackend;
use super::id::Connections;

pub struct EndpointScopeContext<'a, Backend: QuicBackend> {
    pub backend: &'a Backend,
    pub state: &'a mut Backend::EndpointState,
    pub connections: Connections<'a>,
}