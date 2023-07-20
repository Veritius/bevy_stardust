use std::{collections::HashMap, sync::Arc, io};
use log::{error, debug};
use mio::{net::{TcpListener, TcpStream}, Token, Registry, Interest, event::Event};
use rustls::{ServerConfig, ServerConnection};

const LISTENER: Token = Token(0);

pub struct AuthServer {
    server: TcpListener,
    connections: HashMap<Token, OpenConnection>,
    next_id: usize,
    tls_config: Arc<ServerConfig>,
}

impl AuthServer {
    pub fn new(
        server: TcpListener,
        tls_config: Arc<ServerConfig>,
    ) -> Self {
        Self {
            server,
            connections: HashMap::new(),
            next_id: 2,
            tls_config,
        }
    }

    fn accept(
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

    fn connection_event(
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
struct OpenConnection {
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
        if event.is_readable() { self.tls_read(); }
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
enum ConnectionKind {
    Unknown,
    GameClient,
    GameServer,
}