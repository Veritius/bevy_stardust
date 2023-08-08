use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::{mpsc::{Receiver, self, Sender}, Arc, Mutex}, time::{Duration, Instant}, io::{Read, BufRead, Write}, thread, cell::OnceCell};
use bevy::prelude::{Resource, info, error};
use json::{JsonValue, object};
use semver::{Version, VersionReq};

const VERSION_REQ_STR: &'static str = "=0.0.1";
const VERSION_REQ_CELL: OnceCell<VersionReq> = OnceCell::new();
const CONNECTION_TIME_CAP: Duration = Duration::from_secs(30);
const MAX_HICCUPS: u16 = 4;

pub struct TcpListenerServerConfig {
    pub pid: u64,
    pub game_ver: VersionReq,
    pub port: u16,
}

#[derive(Resource)]
pub(super) struct TcpListenerServer(Arc<Mutex<Receiver<TcpListenerMessage>>>);

impl TcpListenerServer {
    pub fn new(config: TcpListenerServerConfig) -> Self {
        let (mut sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let config = config;

            let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
                .expect("TCP listener could not bind to port");

            let srv_pid = format!("{:X}", config.pid);
            let mut clients = Vec::new();
            let mut r_list = vec![];
            
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
                for (idx, client) in clients.iter_mut().enumerate() {
                    if let Ok(bytes) = client.stream.read(&mut buffer) {
                        // Process into JSON
                        let str = String::from_utf8_lossy(&buffer[0..bytes]);
                        process_client(&config, client, &mut sender, &srv_pid, &str);
                    }

                    // Disconnect clients if a shutdown is due
                    if client.shutdown || client.hiccups > MAX_HICCUPS {
                        use std::net::Shutdown;
                        let _ = client.stream.shutdown(Shutdown::Both);
                        r_list.push(idx);
                    }
                }

                // Remove any disconnected clients from the list
                if r_list.len() != 0 {
                    for r in r_list.iter().rev() {
                        clients.remove(*r);
                    }
                    r_list.clear();
                    r_list.shrink_to_fit();
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

    fn send_json_and_hiccup(&mut self, json: JsonValue) {
        self.send_json(json);
        self.hiccups += 1;
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

// Quickly checks the client's data.
fn process_client(
    config: &TcpListenerServerConfig,
    client: &mut WaitingClient,
    sender: &mut Sender<TcpListenerMessage>,
    srv_pid: &str,
    data: &str,
) {
    // Parse JSON
    let json = json::parse(data);
    if json.is_err() {
        client.send_json_and_hiccup(object! { "response": "retry" });
        return;
    }
    let json = json.unwrap();

    // Check the layer version
    let cell = VERSION_REQ_CELL;
    let req = cell.get_or_init(|| { VERSION_REQ_STR.parse::<VersionReq>().unwrap() });
    if !version_comparison(client, req, json["layer_version"].as_str(), "layer") { return };

    // Check the game version
    if !version_comparison(client, &config.game_ver, json["game_version"].as_str(), "game") { return };

    // Check the pid
    fn quick_wrong_pid(client: &mut WaitingClient, srv_pid: &str) { client.send_json_and_close(object! { "response": "wrong_pid", "srv_pid": srv_pid }); }
    if let Some(pid) = json["pid"].as_str() {
        if pid != srv_pid {
            quick_wrong_pid(client, srv_pid);
            return;
        }
    } else {
        quick_wrong_pid(client, srv_pid);
        return;
    }

    sender.send(TcpListenerMessage::ClientAccepted(client.address())).expect("Couldn't communicate over MPSC channel");
    info!("UDP client {}'s connection is accepted", client.address());
}

fn version_comparison(
    client: &mut WaitingClient,
    requirement: &VersionReq,
    version: Option<&str>,
    key: &str,
) -> bool { // returns success
    fn quick_wrong_version(client: &mut WaitingClient, key: &str, requirement: &VersionReq) {
        let version = format!("wrong_{}_version", key);
        let requirement = requirement.to_string();
        client.send_json_and_close(object! { "response": version, "range": requirement });
    }

    if let Some(version) = version {
        if let Ok(version) = version.parse::<Version>() {
            if !requirement.matches(&version) {
                quick_wrong_version(client, key, requirement);
                return false;
            }
        } else {
            quick_wrong_version(client, key, requirement);
            return false;
        }
    } else {
        quick_wrong_version(client, key, requirement);
        return false;
    }

    return true;
}