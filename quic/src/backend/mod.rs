mod connection;
mod endpoint;
mod events;
mod outgoing;
mod protocol;
mod socket;
mod taskpool;

pub(crate) use connection::Handle as ConnectionHandle;
pub(crate) use endpoint::Handle as EndpointHandle;