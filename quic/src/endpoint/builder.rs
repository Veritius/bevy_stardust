use std::marker::PhantomData;
use super::*;

/// A type annotation indicating neither client nor server.
/// Used by [`EndpointBuilder`].
pub struct Dual;

/// A type annotation indicating a server.
/// Used by [`EndpointBuilder`].
pub struct Server;

/// A type annotation indicating a client.
/// Used by [`EndpointBuilder`].
pub struct Client;

mod sealed {
    pub trait Side {}
    impl Side for super::Dual {}
    impl Side for super::Server {}
    impl Side for super::Client {}
}

/// A builder for an [`Endpoint`].
pub struct EndpointBuilder<Side, State>
where
    Side: sealed::Side,
{
    state: State,
    side: PhantomData<Side>,
}

impl<Side, State> EndpointBuilder<Side, State>
where
    Side: sealed::Side,
{
    /// Create an `EndpointBuilder` that can act as both a client and server.
    pub fn dual() -> EndpointBuilder<Dual, ()> {
        todo!()
    }

    /// Create an `EndpointBuilder` for a server.
    pub fn server() -> EndpointBuilder<Server, ()> {
        todo!()
    }

    /// Create an `EndpointBuilder` for a client.
    pub fn client() -> EndpointBuilder<Client, ()> {
        todo!()
    }
}