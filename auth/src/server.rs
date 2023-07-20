use std::{collections::HashMap, sync::Arc};
use mio::{net::{TcpListener, TcpStream}, Token};
use rustls::{ServerConfig, ServerConnection};

const LISTENER: Token = Token(0);

pub struct AuthServer {
    server: TcpListener,
    connections: HashMap<Token, OpenConnection>,
    next_id: usize,
    tls_config: Arc<ServerConfig>,
}

#[derive(Debug)]
struct OpenConnection {
    socket: TcpStream,
    token: Token,
    state: ConnectionState,
    tls_conn: ServerConnection,
    kind: ConnectionKind,
    age: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ConnectionState {
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ConnectionKind {
    GameClient,
    GameServer,
}