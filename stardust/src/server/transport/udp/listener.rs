use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::{mpsc::{Receiver, self}, Arc, Mutex}, time::Duration, io::{Read, BufRead}, thread};
use bevy::prelude::Resource;

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

                    clients.push(WaitingClient {
                        stream,
                        state: WaitingState::RemoveMe,
                    });
                }
            }

            // Process clients
            let mut buffer = [0u8; 1500];
            for client in clients.iter_mut() {
                if let Ok(bytes) = client.stream.read(&mut buffer) {
                    let lines = &buffer[0..bytes-1].lines();
                    

                }
            }
        });

        Self(Arc::new(Mutex::new(receiver)))
    }
}

struct WaitingClient {
    pub stream: TcpStream,
    pub state: WaitingState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WaitingState {
    RemoveMe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TcpListenerMessage {
    ClientAccepted(SocketAddr),
}