use std::{sync::Arc, time::Duration};
use bevy_stardust::channels::ChannelRegistry;

pub use rustls::{self, pki_types::{CertificateDer, PrivateKeyDer}, RootCertStore};

pub use transport::TransportConfig;
pub use endpoint::EndpointConfig;
pub use server::ServerConfig;
pub use client::ClientConfig;

pub mod transport {
    use super::*;

    #[derive(Clone)]
    pub struct TransportConfig {
        pub(crate) quinn: Arc<quinn_proto::TransportConfig>,
    }

    impl TransportConfig {
        pub fn builder() -> TransportConfigBuilder {
            TransportConfigBuilder::new()
        }
    }

    pub struct TransportConfigBuilder {
        quinn: quinn_proto::TransportConfig,
    }

    impl TransportConfigBuilder {
        pub fn new() -> Self {
            Self {
                quinn: quinn_proto::TransportConfig::default(),
            }
        }

        pub fn allow_spin(&mut self, value: bool) -> &mut Self {
            self.quinn.allow_spin(value);
            return self;
        }

        pub fn crypto_buffer_size(&mut self, value: usize) -> &mut Self {
            self.quinn.crypto_buffer_size(value);
            return self;
        }

        pub fn initial_mtu(&mut self, value: u16) -> &mut Self {
            self.quinn.initial_mtu(value.max(1200));
            return self;
        }

        pub fn keep_alive_interval(&mut self, value: Option<Duration>) -> &mut Self {
            self.quinn.keep_alive_interval(value);
            return self;
        }

        pub fn max_idle_timeout(&mut self, value: Option<Duration>) -> &mut Self {
            if let Some(timeout) = value {
                let seconds = timeout.as_secs();

                if seconds > 120 {
                    log::warn!("TransportConfig had its max idle timeout set to {seconds} seconds. This makes the endpoint vulnerable to attacks by malicious actors. Consider reducing the timeout.");
                }
            }

            self.quinn.max_idle_timeout(value.map(|v| {
                quinn_proto::IdleTimeout::try_from(v)
                    .unwrap_or(quinn_proto::IdleTimeout::from(quinn_proto::VarInt::from_u32(30000)))
            }));

            return self;
        }

        pub fn loss_time_threshold(&mut self, value: f32) -> &mut Self {
            self.quinn.time_threshold(value);
            return self;
        }

        pub fn loss_order_threshold(&mut self, value: u32) -> &mut Self {
            self.quinn.packet_threshold(value);
            return self;
        }

        pub fn receive_window(&mut self, value: u64) -> &mut Self {
            self.quinn.receive_window(constrain(value));
            return self;
        }

        pub fn transmit_window(&mut self, value: u64) -> &mut Self {
            self.quinn.send_window(value);
            return self;
        }

        pub fn stream_receive_window(&mut self, value: u64) -> &mut Self {
            self.quinn.stream_receive_window(constrain(value));
            return self;
        }

        pub fn finish(self) -> TransportConfig {
            TransportConfig {
                quinn: Arc::new(self.quinn),
            }
        }
    }

    fn constrain(value: u64) -> quinn_proto::VarInt {
        let value = value.min(2u64.pow(62)-1);
        quinn_proto::VarInt::from_u64(value).unwrap()
    }
}

pub mod endpoint {
    use super::*;

    #[derive(Clone)]
    pub struct EndpointConfig {
        pub(crate) quinn: Arc<quinn_proto::EndpointConfig>,
    }

    impl EndpointConfig {
        pub fn builder() -> EndpointConfigBuilder {
            EndpointConfigBuilder::new()
        }
    }

    pub struct EndpointConfigBuilder {
        quinn: quinn_proto::EndpointConfig,
    }

    impl EndpointConfigBuilder {
        pub fn new() -> Self {
            Self {
                quinn: quinn_proto::EndpointConfig::default(),
            }
        }

        pub fn reset_key(&mut self, key: ring::hmac::Key) -> &mut Self {
            self.quinn.reset_key(Arc::new(key));
            return self;
        }

        pub fn min_reset_interval(&mut self, value: Duration) -> &mut Self {
            self.quinn.min_reset_interval(value);
            return self;
        }

        pub fn allow_grease_bit(&mut self, value: bool) -> &mut Self {
            self.quinn.grease_quic_bit(value);
            return self;
        }

        pub fn finish(self) -> EndpointConfig {
            EndpointConfig {
                quinn: Arc::new(self.quinn),
            }
        }
    }
}

pub mod server {
    use super::*;

    #[derive(Clone)]
    pub struct ServerConfig {
        pub(crate) quinn: Arc<quinn_proto::ServerConfig>,
        pub(crate) transport: Arc<TransportConfig>,

        pub(crate) channels: Arc<ChannelRegistry>,
    }

    impl ServerConfig {
        pub fn builder() -> ServerConfigBuilder<WantsCryptoConfig> {
            ServerConfigBuilder(WantsCryptoConfig { _p: () })
        }
    }

    pub struct ServerConfigBuilder<T>(T);

    pub struct WantsCryptoConfig {
        _p: (),
    }

    impl ServerConfigBuilder<WantsCryptoConfig> {
        pub fn with_single_cert(
            self,
            cert_chain: Vec<CertificateDer<'static>>,
            private_key: PrivateKeyDer<'static>,
        ) -> Result<ServerConfigBuilder<WantsTransportConfig>, rustls::Error> {
            todo!()
        }
    }

    pub struct WantsTransportConfig {
        crypto: Arc<dyn quinn_proto::crypto::ServerConfig>,
    }

    impl ServerConfigBuilder<WantsTransportConfig> {
        pub fn with_transport_config(
            self,
            config: impl Into<Arc<TransportConfig>>,
        ) -> ServerConfigBuilder<WantsChannelRegistry> {
            let config = config.into();

            ServerConfigBuilder(WantsChannelRegistry {
                quinn: {
                    let mut quinn = quinn_proto::ServerConfig::with_crypto(self.0.crypto);
                    quinn.transport = config.quinn.clone();
                    quinn
                },

                transport: config,
            })
        }
    }

    pub struct WantsChannelRegistry {
        quinn: quinn_proto::ServerConfig,
        transport: Arc<TransportConfig>,
    }

    impl ServerConfigBuilder<WantsChannelRegistry> {
        pub fn with_channel_registry(
            self,
            channels: Arc<ChannelRegistry>,
        ) -> ServerConfigBuilder<Ready> {
            ServerConfigBuilder(Ready {
                quinn: self.0.quinn,
                transport: self.0.transport,
                channels,
            })
        }
    }

    pub struct Ready {
        quinn: quinn_proto::ServerConfig,
        transport: Arc<TransportConfig>,
        channels: Arc<ChannelRegistry>,
    }

    impl ServerConfigBuilder<Ready> {
        pub fn finish(self) -> ServerConfig {
            ServerConfig {
                quinn: Arc::new(self.0.quinn),
                transport: self.0.transport,
                channels: self.0.channels,
            }
        }
    }
}

pub mod client {
    use super::*;

    #[derive(Clone)]
    pub struct ClientConfig {
        pub(crate) quinn: quinn_proto::ClientConfig,

        pub(crate) channels: Arc<ChannelRegistry>,
    }
}