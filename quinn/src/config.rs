use std::path::PathBuf;
use rustls::pki_types::{CertificateDer, CertificateRevocationListDer, PrivateKeyDer, TrustAnchor};

#[derive(Debug)]
#[non_exhaustive]
pub enum CertificateOrigin {
    Owned(CertificateDer<'static>),
    File(PathBuf),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PrivateKeyOrigin {
    Owned(PrivateKeyDer<'static>),
    File(PathBuf),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CertificateChainOrigin {
    Owned(Vec<CertificateOrigin>),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CertificateRevocationListOrigin {
    Owned(CertificateRevocationListDer<'static>),
    File(PathBuf),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TrustAnchorOrigin {
    Owned(TrustAnchor<'static>),
    File(PathBuf),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TrustAnchorStoreOrigin {
    Owned(Vec<TrustAnchorOrigin>),
    File(PathBuf),
}

#[derive(Debug)]
pub enum ServerAuthentication {
    Authenticated {
        certificates: CertificateChainOrigin,
        private_key: PrivateKeyOrigin,
    },
}

#[derive(Debug)]
pub enum ClientAuthentication {
    Disabled,
}

/// Configuration for the network thread used to handle communication traffic.
#[derive(Debug)]
pub struct ThreadConfig {
    /// The number of threads dedicated to network traffic.
    /// Must not be zero, or the application will panic.
    pub threads: u32,
}

#[derive(Debug)]
pub struct SocketConfig {
    /// The size of the buffer allocated to receive datagrams.
    /// Higher values allow remote peers to send data more efficiently.
    /// 
    /// The amount of space allocated, in bytes, is equal to the value of this field.
    /// 
    /// If this is set to below `1280`, QUIC packets may be cut off and become unreadable.
    /// Many operating systems also do not buffer UDP datagrams bigger than `65535` bytes,
    /// so setting this field that high may simply waste memory, depending on the operating system.
    pub recv_buf_size: u16,

    /// The size of the buffer allocated to transmit datagrams.
    /// Higher values allow more efficient transmission of information.
    /// 
    /// The amount of space allocated, in bytes, is equal to the value of this field.
    /// 
    /// If this is set to below `1280`, QUIC packets may be cut off and become unreadable.
    /// Many operating systems also do not buffer UDP datagrams bigger than `65535` bytes,
    /// so setting this field that high may simply waste memory, depending on the operating system.
    pub send_buf_size: u16,
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            recv_buf_size: 1478,
            send_buf_size: 1478,
        }
    }
}