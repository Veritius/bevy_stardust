use std::{net::SocketAddr, sync::Arc};
use bevy_ecs::{prelude::*, system::EntityCommands};

pub trait EndpointCommands {
    fn make_endpoint(
        &mut self,
        config: EndpointConfig,
        build: impl FnOnce(Result<EndpointBuilder, EndpointBuildError>),
    ) -> &mut Self;

    fn close_endpoint(
        &mut self,
    ) -> &mut Self;
}

#[derive(Clone)]
pub struct EndpointConfig {
    pub address: SocketAddr,
    pub quinn: Arc<quinn_proto::EndpointConfig>,
    pub server: Option<Arc<quinn_proto::ServerConfig>>,
}

#[derive(Debug, Clone)]
pub enum EndpointBuildError {

}

impl<'w> EndpointCommands for EntityWorldMut<'w> {
    fn make_endpoint(
        &mut self,
        config: EndpointConfig,
        build: impl FnOnce(Result<EndpointBuilder, EndpointBuildError>),
    ) -> &mut Self {
        todo!()
    }

    fn close_endpoint(
        &mut self,
    ) -> &mut Self {
        todo!()
    }
}

impl<'w> EndpointCommands for EntityCommands<'w> {
    fn make_endpoint(
        &mut self,
        config: EndpointConfig,
        build: impl FnOnce(Result<EndpointBuilder, EndpointBuildError>),
    ) -> &mut Self {
        todo!()
    }

    fn close_endpoint(
        &mut self,
    ) -> &mut Self {
        todo!()
    }
}

pub struct EndpointBuilder<'a> {
    commands: &'a mut quinn_proto::Endpoint,
}

impl<'a> EndpointBuilder<'a> {
    pub fn connect(
        &mut self,
        config: ClientConfig,
        build: impl FnOnce(Result<ConnectionBuilder, ConnectionBuildError>),
    ) {
        todo!()
    }
}

pub struct ConnectionBuilder<'a> {
    commands: EntityCommands<'a>,
}

#[derive(Clone)]
pub struct ClientConfig {
    pub quinn: quinn_proto::ClientConfig,
}

#[derive(Debug, Clone)]
pub enum ConnectionBuildError {

}