//! Encryption functionality for the transport layer.

pub(crate) mod settings;

pub use settings::{ClientTlsConfig, ServerTlsConfig};

pub use rustls::{RootCertStore, pki_types::{CertificateDer, PrivateKeyDer}};
pub use rustls_pemfile::certs as extract_certs;
pub use rustls_pemfile::private_key as extract_key;