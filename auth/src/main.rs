pub mod certs;

use std::{collections::HashMap, sync::Arc};
use certs::server_config;
use clap::Parser;
use mio::{net::{TcpStream, TcpListener}, Token};
use rustls::{ServerConnection, ServerConfig};

const LISTENER: Token = Token(0);

fn main() {
    let args = CommandLineArguments::parse();

    let listener = TcpListener::bind(args.address.parse().expect("IP could not be parsed")).expect("Couldn't bind to IP");
    let config = server_config(&args);
    
    let mut server = StardustAuthServer::new(listener, config);
}

#[derive(Parser, Debug)]
struct CommandLineArguments {
    /// Your game's protocol ID.
    #[arg(short, long)]
    protocol: u32,

    /// IP address/port for the server to bind to. Supports IPv6.
    #[arg(short, long, default_value="0.0.0.0:24060")]
    address: String,

    /// Path for the certificate file, relative to the executable.
    #[arg(short, long)]
    cert_path: String,
}

struct StardustAuthServer {
    server: TcpListener,
    config: Arc<ServerConfig>,
    connections: HashMap<Token, OpenConnection>,
    next_id: usize,
}

impl StardustAuthServer {
    pub fn new(server: TcpListener, config: Arc<ServerConfig>) -> Self {
        Self {
            server,
            config,
            connections: HashMap::new(),
            next_id: 0,
        }
    }
}

struct OpenConnection {
    socket: TcpStream,
    token: Token,
    source: ConnectionSource,
    tls: ServerConnection,
}

enum ConnectionSource {
    GameClient,
    GameServer,
}