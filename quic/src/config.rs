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
        quinn: Arc<quinn_proto::TransportConfig>,
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
        quinn: Arc<quinn_proto::EndpointConfig>,
    }
}

pub mod server {
    use super::*;

    #[derive(Clone)]
    pub struct ServerConfig {
        quinn: Arc<quinn_proto::ServerConfig>,

        channels: Arc<ChannelRegistry>,
    }    
}

pub mod client {
    use super::*;

    #[derive(Clone)]
    pub struct ClientConfig {
        quinn: quinn_proto::ClientConfig,

        channels: Arc<ChannelRegistry>,
    }
}