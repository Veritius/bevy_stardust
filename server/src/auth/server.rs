use std::{sync::{mpsc::{self, Receiver}, Arc, Mutex}, thread, collections::HashMap, net::SocketAddr};
use bevy::prelude::Resource;
use bevy_stardust_shared::rustls::{ServerConfig, Certificate, PrivateKey};
use mio::{net::{TcpListener, TcpStream}, Token};
use super::tlsconfig::server_config;

#[derive(Resource)]
pub struct AuthenticatorServer {
    // The receiver will only be read by one system, so this is probably fine.
    pub receiver: Arc<Mutex<Receiver<AuthenticatorResponse>>>,
}

pub enum AuthenticatorResponse {
    ClientAccepted,
}

#[must_use]
pub fn start_auth_server(
    address: impl Into<SocketAddr>,
    certificates: Vec<Certificate>,
    private_key: PrivateKey,
) -> AuthenticatorServer {
    // Establish channel between auth and main thread
    let channel = mpsc::channel();
    let address = address.into();

    // Start thread
    thread::spawn(move || {
        let sender = channel.0;
        let server = AuthenticatorServerInternal {
            listener: TcpListener::bind(address).expect(&format!("Auth server could not bind to {:?}", address)),
            connections: HashMap::new(),
            next_id: 0,
            tls_config: server_config(
                certificates,
                private_key
            ),
        };
    });

    AuthenticatorServer {
        receiver: Arc::new(Mutex::new(channel.1))
    }
}

pub(super) struct AuthenticatorServerInternal {
    listener: TcpListener,
    connections: HashMap<Token, OpenConnection>,
    next_id: usize,
    tls_config: Arc<ServerConfig>,
}

pub(super) struct OpenConnection {
    socket: TcpStream,
    token: Token,
}