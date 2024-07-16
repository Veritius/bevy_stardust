use anyhow::Result;
use boring::ssl::{SslContextBuilder, SslMethod};
use quiche::Config;
use crate::endpoint::*;

pub(crate) fn build_client(state: ClientReady) -> Result<Endpoint> {
    // Setup BoringSSL's SSL stuff
    let ssl = setup_ssl_shared(&state.shared)?;
    let ssl = setup_ssl_join(ssl, &state.join)?;

    // Quiche config object
    let config = setup_config_shared(ssl, &state.shared)?;

    todo!()
}

pub(crate) fn build_server(state: ServerReady) -> Result<Endpoint> {
    // Setup BoringSSL's SSL stuff
    let ssl = setup_ssl_shared(&state.shared)?;
    let ssl = setup_ssl_host(ssl, &state.host)?;

    // Quiche config object
    let config = setup_config_shared(ssl, &state.shared)?;

    todo!()
}

pub(crate) fn build_dual(state: DualReady) -> Result<Endpoint> {
    // Setup BoringSSL's SSL stuff
    let ssl = setup_ssl_shared(&state.shared)?;
    let ssl = setup_ssl_host(ssl, &state.host)?;
    let ssl = setup_ssl_join(ssl, &state.join)?;

    // Quiche config object
    let config = setup_config_shared(ssl, &state.shared)?;

    todo!()
}

fn setup_ssl_shared(
    shared: &ReadyShared,
) -> Result<SslContextBuilder> {
    // TODO: Only allow TLS 1.3
    let mut ssl = SslContextBuilder::new(SslMethod::tls())?;

    // Set the trust anchors
    ssl.set_cert_store(todo!() /* shared.anchors.into_boring_x509_store() */);

    return Ok(ssl);
}

fn setup_ssl_host(
    mut ssl: SslContextBuilder,
    host: &HostShared,
) -> Result<SslContextBuilder> {
    // Private key
    ssl.set_private_key(host.credentials.private_key.as_boring_pkey_ref())?;

    // First certificate in chain
    let mut iter = host.credentials.certificates.iter();
    ssl.set_certificate(iter.next().unwrap().as_boring_x509_ref())?;

    // The rest of the chain
    for cert in iter {
        ssl.add_extra_chain_cert(cert.as_boring_x509())?;
    }

    // Return builder
    return Ok(ssl);
}

fn setup_ssl_join(
    mut ssl: SslContextBuilder,
    join: &JoinShared,
) -> Result<SslContextBuilder> {
    return Ok(ssl);
}

fn setup_config_shared(
    ssl: SslContextBuilder,
    shared: &ReadyShared,
) -> Result<Config> {
    // Create the config object
    let mut config = Config::with_boring_ssl_ctx_builder(quiche::PROTOCOL_VERSION, ssl)?;

    // Set the application protos
    config.set_application_protos(todo!())?;

    // Enable datagrams (for unreliable traffic)
    config.enable_dgram(true, todo!(), todo!());

    // Return the config
    return Ok(config);
}