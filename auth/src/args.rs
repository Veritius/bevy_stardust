use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// The TCP port the server will bind to.
    #[arg(short='p', long="port")]
    pub tcp_port: u16,

    /// The certificate file for the TLS server.
    #[arg(short='c', long="cert")]
    pub cert_path: String,

    /// The private key for the TLS server.
    #[arg(short='k', long="key")]
    pub key_path: String,
}