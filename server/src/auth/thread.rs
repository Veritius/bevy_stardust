use std::{sync::mpsc::{self, Receiver}, thread};

pub fn start_auth_server() -> AuthenticatorServer {
    // Establish channel between auth and main thread
    let channel = mpsc::channel();

    // Start thread
    thread::spawn(move || {
        let sender = channel.0;
    });

    AuthenticatorServer { receiver: channel.1 }
}

pub struct AuthenticatorServer {
    pub receiver: Receiver<AuthenticatorResponse>,
}

pub enum AuthenticatorResponse {
    ClientAccepted,
}