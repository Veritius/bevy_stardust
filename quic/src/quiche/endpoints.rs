// use anyhow::Result;
// use boring::ssl::{SslContextBuilder, SslMethod, SslVersion};
// use quiche::Config;
// use crate::{endpoint::*, AppProtos, TrustAnchors};

// pub(crate) fn build_client(state: ClientReady) -> Result<EndpointShared> {
//     // Setup BoringSSL's SSL stuff
//     let ssl = setup_ssl_shared(state.shared.anchors)?;
//     let ssl = setup_ssl_join(ssl, &state.join)?;

//     // Quiche config object
//     let quiche_config = setup_config_shared(ssl, state.shared.protos)?;

//     // Create component
//     return Ok(EndpointShared {
//         listening: false,
//         send_size: 1280,
//         recv_size: 1472,
//         socket: state.shared.socket,
//         connections: EndpointConnections::new(),
//         quiche_config,
//     });
// }

// pub(crate) fn build_server(state: ServerReady) -> Result<EndpointShared> {
//     // Setup BoringSSL's SSL stuff
//     let ssl = setup_ssl_shared(state.shared.anchors)?;
//     let ssl = setup_ssl_host(ssl, &state.host)?;

//     // Quiche config object
//     let quiche_config = setup_config_shared(ssl, state.shared.protos)?;

//     // Create component
//     return Ok(EndpointShared {
//         listening: true,
//         send_size: 1280,
//         recv_size: 1472,
//         socket: state.shared.socket,
//         connections: EndpointConnections::new(),
//         quiche_config,
//     });
// }

// pub(crate) fn build_dual(state: DualReady) -> Result<EndpointShared> {
//     // Setup BoringSSL's SSL stuff
//     let ssl = setup_ssl_shared(state.shared.anchors)?;
//     let ssl = setup_ssl_host(ssl, &state.host)?;
//     let ssl = setup_ssl_join(ssl, &state.join)?;

//     // Quiche config object
//     let quiche_config = setup_config_shared(ssl, state.shared.protos)?;

//     // Create component
//     return Ok(EndpointShared {
//         listening: false,
//         send_size: 1280,
//         recv_size: 1472,
//         socket: state.shared.socket,
//         connections: EndpointConnections::new(),
//         quiche_config,
//     });
// }

// fn setup_ssl_shared(
//     anchors: TrustAnchors,
// ) -> Result<SslContextBuilder> {
//     // Setup the BoringSSL context
//     let mut ssl = SslContextBuilder::new(SslMethod::tls())?;
//     ssl.set_min_proto_version(Some(SslVersion::TLS1_3))?;

//     // Set the trust anchors
//     ssl.set_cert_store(anchors.into_boring_x509_store()?);

//     return Ok(ssl);
// }

// fn setup_ssl_host(
//     mut ssl: SslContextBuilder,
//     host: &HostShared,
// ) -> Result<SslContextBuilder> {
//     // Private key
//     ssl.set_private_key(host.credentials.private_key.as_boring_pkey_ref())?;

//     // First certificate in chain
//     let mut iter = host.credentials.certificates.iter();
//     ssl.set_certificate(iter.next().unwrap().as_boring_x509_ref())?;

//     // The rest of the chain
//     for cert in iter {
//         ssl.add_extra_chain_cert(cert.as_boring_x509())?;
//     }

//     // Check the private key
//     ssl.check_private_key()?;

//     // Return builder
//     return Ok(ssl);
// }

// fn setup_ssl_join(
//     mut ssl: SslContextBuilder,
//     join: &JoinShared,
// ) -> Result<SslContextBuilder> {
//     return Ok(ssl);
// }

// fn setup_config_shared(
//     ssl: SslContextBuilder,
//     protos: AppProtos,
// ) -> Result<Config> {
//     // Create the config object
//     let mut config = Config::with_boring_ssl_ctx_builder(quiche::PROTOCOL_VERSION, ssl)?;

//     // Set the application protos
//     config.set_application_protos(&protos.collect())?;

//     // Enable datagrams (for unreliable traffic)
//     // TODO: Make these queue length values set by the user
//     config.enable_dgram(true, 65535, 4096);

//     // Return the config
//     return Ok(config);
// }

use std::{collections::VecDeque, net::SocketAddr};
use bevy::utils::HashMap;
use bytes::Bytes;
use crate::endpoint::Transmit;

pub struct QuicheEndpoint {
    config: quiche::Config,

    incoming: HashMap<SocketAddr, VecDeque<Bytes>>,
    outgoing: HashMap<SocketAddr, VecDeque<Bytes>>,
}

impl crate::endpoint::EndpointState for QuicheEndpoint {
    type Backend = super::Quiche;
    type IoError = quiche::Error;

    fn recv_udp_packet(&mut self, from: std::net::SocketAddr, packet: &[u8]) -> Result<(), Self::IoError> {
        if packet.len() < 1280 { return Err(quiche::Error::InvalidPacket); }

        self.incoming
            .entry(from)
            .or_insert_with(|| VecDeque::new())
            .push_back(Bytes::copy_from_slice(packet));

        return Ok(());
    }

    fn send_udp_packet(&mut self) -> impl Iterator<Item = Result<Transmit, Self::IoError>> + '_ {
        self.outgoing
            .iter_mut()
            .flat_map(|(addr, vec)| vec.drain(..).map(|v| (*addr, v)))
            .map(|(remote, data)| {
                let tk: Result<Transmit, Self::IoError> = Ok(Transmit { remote, data });
                tk
            })
    }
}