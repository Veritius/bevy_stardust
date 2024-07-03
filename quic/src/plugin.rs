use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connections::*;
use crate::endpoints::*;

/// Adds QUIC functionality to the application.
pub struct QuicPlugin;

impl Plugin for QuicPlugin {
    fn build(&self, app: &mut App) {
        if app.world.get_resource::<QuicConfig>().is_none() {
            app.init_resource::<QuicConfig>();
        }

        app.add_systems(PreUpdate, (
            endpoint_datagram_recv_system,
            connection_endpoint_events_system,
            connection_dump_pending_system,
            connection_event_handler_system,
            connection_update_rtt_system,
        ).chain().in_set(NetworkRecv::Receive));

        app.add_systems(PostUpdate, (
            connection_disconnect_system,
            connection_message_sender_system,
            connection_datagram_send_system,
            connection_endpoint_events_system,
        ).chain().in_set(NetworkSend::Transmit));

        app.add_systems(Last, quic_config_checker_system);
    }
}

/// Configuration used by systems added by [`QuicPlugin`].
#[derive(Debug, Resource)]
pub struct QuicConfig {
    /// The maximum size of a **UDP datagram**, used to allocate space for I/O.
    /// 
    /// This defaults to `1472` bytes, the maximum size of an Ethernet packet.
    /// This also plugin does automatic MTU detection if enabled on the `Endpoint`.
    /// The minimum possible value is `1280`, imposed by [RFC 9000] as part of the QUIC protocol.
    /// The highest you will ever realistically see is `2^16` (`65_535`) with IP jumbo frames.
    /// 
    /// The amount of memory allocated (in bytes) can be calculated with `min(t,e) * n`
    /// where `t` is the number of threads used by Bevy for tasks, and `e` is the number of endpoints,
    /// and `n` is the `maximum_transport_units` value.
    /// 
    /// [RFC 9000]: https://www.rfc-editor.org/rfc/rfc9000.html
    pub maximum_transport_units: usize,

    /// The maximum length of framed messages, applying to Stardust messages.
    /// This imposes a maximum limit to the length of a single Stardust message sent over the connection.
    /// 
    /// Defaults to `2^16` (`65_535`) bytes.
    /// 
    /// # Oversize frame attack
    /// When a framed message is sent, it's headed with a number indicating its length.
    /// This is used to buffer the frame in memory in case it's not sent in a single packet.
    /// However, by setting the number to something ridiculously high like `2^62` (the hard maximum),
    /// a malicious actor could perform a Denial of Service attack by running the server out of memory.
    /// 
    /// This value addresses this (for individual messages) by introducing a limit to this length value.
    /// Length values over the maximum will be rejected, and will cause the peer to be disconnected.
    /// To make sure your peers aren't falsely disconnected, this should be set to a value high enough
    /// that your program will realistically never reach it, plus some wiggle room.
    pub maximum_framed_message_length: usize,

    /// The maximum amount of buffered data that can be stored for framed messages.
    /// This imposes a maximum limit to the total amount of data in memory for unfinished messages.
    /// 
    /// Defaults to `2^18` (`262_144`) bytes.
    /// 
    /// # Memory usage attack
    /// When a portion of a framed message is sent, it's stored in memory until the
    /// rest is received, at which point it's pushed to the Stardust message queue (`NetworkMessages<Incoming>`). 
    /// However, due to this approach, another denial of service attack becomes possible.
    /// By sending many very large, but not large enough to trip the individual message length threshold,
    /// while also never finishing any of these fragmented messages, malicious actors can waste a lot of memory.
    /// 
    /// This variable addresses this attack by having an upper limit to the amount of data that
    /// each peer is permitted to allocate. It's also related to [`maximum_framed_message_length`][mfml],
    /// as this check will trip if the threshold is set less than `maximum_framed_message_length`.
    /// 
    /// [mfml]: Self::maximum_framed_message_length
    pub maximum_buffered_frame_data: usize,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            maximum_transport_units: 1472_usize,
            maximum_framed_message_length: 2_usize.pow(16),
            maximum_buffered_frame_data: 2_usize.pow(18),
        }
    }
}

impl QuicConfig {
    /// The minimum MTU value that is permitted.
    pub const MINIMUM_MTU: usize = 1280;
}

fn quic_config_checker_system(
    mut config: ResMut<QuicConfig>,
) {
    if !config.is_changed() { return }

    { // MTU checks start
        let mtu = config.maximum_transport_units;

        if mtu < QuicConfig::MINIMUM_MTU {
            error!("MTU value ({mtu}) was below QUIC minimum");
            config.maximum_transport_units = QuicConfig::MINIMUM_MTU;
        }
    
        if mtu > 65535 {
            warn!("MTU value ({mtu}) is greater than any packet that will ever be sent");
            config.maximum_transport_units = 65535;
        }
    } // MTU checks end

    { // Frame len checks start
        let (mfml, mbfd) = (config.maximum_framed_message_length, config.maximum_buffered_frame_data);
        
        if mfml > mbfd {
            warn!("Maximum buffered data value ({mbfd}) was less than maximum framed message length ({mfml})");
        }
    } // Frame len checks end
}