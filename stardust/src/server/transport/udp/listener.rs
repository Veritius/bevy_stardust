use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::{mpsc::{Receiver, self}, Arc, Mutex}, time::Duration, io::{Read, BufRead, Write}, thread, cell::OnceCell};
use bevy::prelude::{Resource, info};
use json::{JsonValue, object};
use semver::{Version, VersionReq};

const VERSION_REQ_STR: &'static str = "=0.0.1";
const VERSION_REQ_CELL: OnceCell<VersionReq> = OnceCell::new();
const MAX_HICCUPS: u16 = 16;

#[derive(Resource)]
pub(super) struct TcpListenerServer(Arc<Mutex<Receiver<TcpListenerMessage>>>);

impl TcpListenerServer {
    pub fn new(pid: u64, port: u16) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                .expect("TCP listener could not bind to port");

            let pid = format!("{:X}", pid);
            let mut clients = Vec::new();
            
            loop {
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
                            state: ClientState::WaitingInitial,
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

                        process_client(client, json)
                    }
                }
            }
        });

        Self(Arc::new(Mutex::new(receiver)))
    }
}

struct WaitingClient {
    pub stream: TcpStream,
    pub hiccups: u16,
    pub state: ClientState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClientState {
    WaitingInitial,
    Accepted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TcpListenerMessage {
    ClientAccepted(SocketAddr),
}

fn process_client(
    client: &mut WaitingClient,
    json: Result<JsonValue, json::Error>
) {
    if json.is_err() { todo!() }
    let json = json.unwrap();

    match client.state {
        ClientState::WaitingInitial => {
            let mut json_response = object!{};

            let version = json["version"].as_str();
            let pid = json["pid"].as_str();
        },
        ClientState::Accepted => panic!("Client was in Accepted state and was processed"),
    }
}