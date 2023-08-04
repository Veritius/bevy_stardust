use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::{mpsc::{Receiver, self}, Arc, Mutex}, time::Duration, io::{Read, BufRead, Write}, thread};
use bevy::prelude::{Resource, info};
use self::error::json_error;

mod error;

/// Maximum mistakes a client can make before they are terminated.
const HICCUP_DISCONNECT_THRESHOLD: u8 = 12;

#[derive(Resource)]
pub(super) struct TcpListenerServer(Arc<Mutex<Receiver<TcpListenerMessage>>>);

impl TcpListenerServer {
    pub fn new(port: u16) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                .expect("TCP listener could not bind to port");

            let mut clients = Vec::new();

            // Accept all incoming
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    // Configure stream
                    stream.set_nonblocking(true).unwrap();
                    stream.set_read_timeout(Some(Duration::from_secs_f32(5.0))).unwrap();

                    info!("Accepted TCP connection from address {}", stream.peer_addr().unwrap());

                    clients.push(WaitingClient {
                        stream,
                        hiccups: 0,
                        state: WaitingState::JustConnected,
                    });
                }
            }

            // Process clients
            let mut buffer = [0u8; 1500];
            for client in clients.iter_mut() {
                if let Ok(bytes) = client.stream.read(&mut buffer) {
                    // Process into JSON
                    let str = String::from_utf8_lossy(&buffer[0..bytes]);
                    let json = json::parse(&str);

                    // Respond with packet and mutate client obj
                    match json {
                        Ok(json) => todo!(),
                        Err(err) => { json_error(client, err) },
                    }

                    if client.hiccups > HICCUP_DISCONNECT_THRESHOLD {
                        todo!()
                    }
                }
            }
        });

        Self(Arc::new(Mutex::new(receiver)))
    }
}

struct WaitingClient {
    pub stream: TcpStream,
    pub hiccups: u8,
    pub state: WaitingState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WaitingState {
    JustConnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TcpListenerMessage {
    ClientAccepted(SocketAddr),
}