use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::{mpsc::{Receiver, self, Sender}, Arc, Mutex}, time::{Duration, Instant}, io::{Read, BufRead, Write}, thread, cell::OnceCell};
use bevy::prelude::{Resource, info, error};
use json::{JsonValue, object};
use semver::{Version, VersionReq};

const VERSION_REQ_STR: &'static str = "=0.0.1";
const VERSION_REQ_CELL: OnceCell<VersionReq> = OnceCell::new();
const CONNECTION_TIME_CAP: Duration = Duration::from_secs(30);

#[derive(Resource)]
pub(super) struct TcpListenerServer(Arc<Mutex<Receiver<TcpListenerMessage>>>);

impl TcpListenerServer {
    pub fn new(pid: u64, port: u16) -> Self {
        let (mut sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                .expect("TCP listener could not bind to port");

            let srv_pid = format!("{:X}", pid);
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
                            connected: Instant::now(),
                            hiccups: 0,
                            shutdown: false,
                        });
                    }
                }

                // Process clients
                let mut buffer = [0u8; 1500];
                for client in clients.iter_mut() {
                    if let Ok(bytes) = client.stream.read(&mut buffer) {
                        // Process into JSON
                        let str = String::from_utf8_lossy(&buffer[0..bytes]);

                        process_client(client, &mut sender, &srv_pid, &str);
                    }
                }
            }
        });

        Self(Arc::new(Mutex::new(receiver)))
    }
}

struct WaitingClient {
    pub stream: TcpStream,
    pub connected: Instant,
    pub hiccups: u16,
    pub shutdown: bool,
}

impl WaitingClient {
    fn address(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    fn send_json(&mut self, json: JsonValue) {
        let _ = self.stream.write(json.dump().as_bytes());
        if let Err(err) = self.stream.flush() {
            error!("Encountered error sending TCP packet to remote peer: {}", err);
        };
    }

    fn send_json_and_close(&mut self, json: JsonValue) {
        self.send_json(json);
        self.shutdown = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TcpListenerMessage {
    ClientAccepted(SocketAddr),
}

fn process_client(
    client: &mut WaitingClient,
    sender: &mut Sender<TcpListenerMessage>,
    srv_pid: &str,
    data: &str,
) {
    // Parse JSON
    let json = json::parse(data);
    if json.is_err() {
        client.send_json(object! { "response": "retry" });
        client.hiccups += 1;
    }
    
    let json = json.unwrap();

    // Check the version
    if let Some(version) = json["version"].as_str() {
        
    } else {
        client.send_json_and_close(object! { "response": "wrong_version", "range": "todo" });
    }

    // Check the pid
    if let Some(pid) = json["pid"].as_str() {

    } else {
        client.send_json_and_close(object! { "response": "wrong_pid", "srv_pid": srv_pid });
    }

    sender.send(TcpListenerMessage::ClientAccepted(client.address())).expect("Couldn't communicate over MPSC channel");
}