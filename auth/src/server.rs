use std::{collections::HashMap, sync::Arc, io::{self, Read, Write}, path::PathBuf, net::SocketAddr, str::FromStr};
use log::{error, debug};
use mio::{net::{TcpListener, TcpStream}, Token, Registry, Interest, event::Event, Poll};
use rustls::{ServerConfig, ServerConnection};
use crate::{config::ConfigFile, crypto::{load_certificates, load_private_key}, msg::parse_message};

pub const LISTENER: Token = Token(0);

pub fn setup_server(config: &ConfigFile) -> (AuthServer, Poll) {
    let certificates = load_certificates(PathBuf::from_str(&config.encryption.certificate).unwrap().as_path());
    let privatekey = load_private_key(PathBuf::from_str(&config.encryption.privatekey).unwrap().as_path());

    let server_config = Arc::new(ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates, privatekey)
        .expect("Private key was invalid"));

    let mut listener = TcpListener::bind(SocketAddr::from_str(&format!("0.0.0.0:{}", config.server.port)).expect("Invalid port value"))
        .expect("Couldn't bind to address (port already occupied?)");
    let poll = Poll::new().unwrap();
    poll.registry().register(&mut listener, LISTENER, Interest::READABLE).unwrap();

    (AuthServer::new(listener, server_config), poll)
}

pub struct AuthServer {
    server: TcpListener,
    connections: HashMap<Token, OpenConnection>,
    next_id: usize,
    tls_config: Arc<ServerConfig>,
}

impl AuthServer {
    pub fn new(
        listener: TcpListener,
        tls_config: Arc<ServerConfig>,
    ) -> Self {
        Self {
            server: listener,
            connections: HashMap::new(),
            next_id: 2,
            tls_config,
        }
    }

    pub fn accept(
        &mut self,
        registry: &Registry,
    ) -> Result<(), io::Error> {
        loop {
            match self.server.accept() {
                Ok((socket, address)) => {
                    debug!("Accepting connection from {:?}", address);

                    let tls_conn = ServerConnection::new(Arc::clone(&self.tls_config)).unwrap();
                    let token = Token(self.next_id);
                    self.next_id += 1;

                    let mut connection = OpenConnection::new(socket, token, tls_conn);
                },
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(err) => {
                    error!("Error while accepting connection: {:?}", err);
                    return Err(err);
                }
            }
        }
    }

    pub fn connection_event(
        &mut self,
        registry: &Registry,
        event: &Event,
    ) {
        let token = event.token();
        if self.connections.contains_key(&token) {
            self.connections
                .get_mut(&token)
                .unwrap()
                .ready(registry, event);

            if self.connections[&token].state() == ConnectionState::Closed {
                self.connections.remove(&token);
            }
        }
    }
}

#[derive(Debug)]
pub struct OpenConnection {
    socket: TcpStream,
    token: Token,
    state: ConnectionState,
    tls_conn: ServerConnection,
    kind: ConnectionKind,
    age: f32,
}

impl OpenConnection {
    fn new(
        socket: TcpStream,
        token: Token,
        tls_conn: ServerConnection,
    ) -> Self {
        Self {
            socket,
            token,
            state: ConnectionState::Open,
            tls_conn,
            kind: ConnectionKind::Unknown,
            age: 0.0,
        }
    }

    fn ready(
        &mut self,
        registry: &Registry,
        event: &Event,
    ) {
        if event.is_readable() {
            self.tls_read();
            if let Ok(Some(string)) = self.plain_read() {
                parse_message(self, &string);
            }
        }
        if event.is_writable() { self.tls_write_and_handle_error(); }

        if self.state() == ConnectionState::Closing {
            let _ = self.socket.shutdown(std::net::Shutdown::Both);
            self.state = ConnectionState::Closed;
            self.deregister(registry);
        } else {
            self.reregister(registry);
        }
    }

    fn tls_read(&mut self) {
        match self.tls_conn.read_tls(&mut self.socket) {
            Err(err) => {
                if err.kind() == io::ErrorKind::WouldBlock { return; }
                error!("Error while reading TLS {:?}", err);
                self.state = ConnectionState::Closing;
                return;
            }
            Ok(0) => {
                debug!("Encountered end of field");
                self.state = ConnectionState::Closing;
                return;
            }
            Ok(_) => {}
        }

        if let Err(err) = self.tls_conn.process_new_packets() {
            error!("Couldn't process packet: {:?}", err);
            self.tls_write_and_handle_error();
            self.state = ConnectionState::Closing;
        }
    }

    fn tls_write(&mut self) -> io::Result<usize> {
        self.tls_conn.write_tls(&mut self.socket)
    }

    fn tls_write_and_handle_error(&mut self) {
        if let Err(err) = self.tls_write() {
            error!("Failed to write TLS message: {:?}", err);
            self.state = ConnectionState::Closing;
        }
    }

    /// Read incoming plaintext for this connection. Use `tls_read` before this.
    fn plain_read(&mut self) -> Result<Option<String>, ()> {
        if let Ok(io_state) = self.tls_conn.process_new_packets() {
            if io_state.plaintext_bytes_to_read() == 0 { return Ok(None); }
            let mut buf = Vec::new();
            buf.resize(io_state.plaintext_bytes_to_read(), 0u8);
            self.tls_conn.reader().read_exact(&mut buf).unwrap();
            let m = String::from_utf8(buf);
            if m.is_err() { return Err(()); }
            return Ok(Some(m.unwrap()));
        } else {
            return Err(())
        }
    }

    /// Write plaintext to this connection. Use `tls_write` to send an encrypted message.
    pub fn plain_write(&mut self, buf: &[u8]) {
        self.tls_conn.writer().write_all(buf).unwrap();
    }

    fn register(
        &mut self,
        registry: &Registry,
    ) {
        let interests = self.interests();
        registry
            .register(&mut self.socket, self.token, interests)
            .unwrap();
    }

    fn reregister(
        &mut self,
        registry: &Registry,
    ) {
        let interests = self.interests();
        registry
            .reregister(&mut self.socket, self.token, interests)
            .unwrap();
    }

    fn deregister(
        &mut self,
        registry: &Registry,
    ) {
        registry
            .deregister(&mut self.socket)
            .unwrap();
    }

    fn interests(&self) -> Interest {
        match (
            self.tls_conn.wants_read(),
            self.tls_conn.wants_write()
        ) {
            (true, true) => { Interest::READABLE | Interest::WRITABLE },
            (false, true) => { Interest::WRITABLE },
            _ => { Interest::READABLE},
        }
    }

    fn state(&self) -> ConnectionState {
        self.state
    }

    pub fn kind(&self) -> ConnectionKind {
        self.kind
    }

    // fn tick_age(
    //     &mut self,
    //     args: &Args,
    //     seconds: f32,
    // ) -> bool {
    //     self.age += seconds;
    //     return seconds > args.terminate_time;
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ConnectionState {
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionKind {
    Unknown,
    GameClient(ClientConnectionStage),
    GameServer(ServerConnectionStage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClientConnectionStage {
    WaitingOnServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]

pub enum ServerConnectionStage {
    ReplaceMe,
}