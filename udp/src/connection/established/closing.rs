use std::time::Instant;
use bevy::ecs::entity::EntityHashSet;
use bevy_stardust::prelude::*;
use control::ControlFrameIdent;
use frames::frames::{FrameFlags, FrameType, SendFrame};
use crate::prelude::*;
use super::*;

#[derive(Debug)]
pub(in crate::connection) struct Closing {
    pub finished: bool,
    pub informed: bool,
    pub origin: CloseOrigin,
    pub reason: Option<Bytes>,
}

impl Closing {
    pub(super) fn new(
        origin: CloseOrigin,
        reason: Option<Bytes>,
    ) -> Self {
        Self {
            finished: false,
            informed: false,
            origin,
            reason,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CloseOrigin {
    Local,
    Remote,
}

#[derive(Event)]
pub(in crate::connection) struct DisconnectEstablishedPeerEvent {
    inner: DisconnectPeerEvent,
}

pub(in crate::connection) fn established_close_events_system(
    mut transport_events: EventReader<DisconnectEstablishedPeerEvent>,
    mut stardust_events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<(Entity, &mut Established, Option<&mut NetworkPeerLifestage>), With<Connection>>,
) {
    let mut processed_map = EntityHashSet::default();
    let iter = transport_events.read().map(|f| &f.inner).chain(stardust_events.read());
    for event in iter {
        if processed_map.contains(&event.peer) { continue }
        processed_map.insert(event.peer);

        // The error case means that the entity is a network peer we don't control
        // Or, the peer entity was spuriously deleted without going through the right steps
        if let Ok((entity, mut established, lifestage)) = connections.get_mut(event.peer) {
            if established.closing.is_some() { continue } // Already closing
            debug!("Closing connection with {entity:?}");

            established.closing = Some(Closing::new(CloseOrigin::Local, event.reason.clone()));
            established.builder.put(SendFrame {
                priority: u32::MAX,
                time: Instant::now(),
                flags: FrameFlags::IDENTIFIED,
                ftype: FrameType::Control,
                reliable: true,
                order: None,
                ident: Some(ControlFrameIdent::BeginClose.into()),
                payload: Bytes::new(),
            });

            if let Some(mut lifestage) = lifestage {
                *lifestage = NetworkPeerLifestage::Closing;
            }
        }
    }
}

pub(in crate::connection) fn established_closing_write_system(
    mut connections: Query<&mut Established, With<Connection>>,
) {
    connections.par_iter_mut().for_each(|mut established| {
        if let Some(closing) = &mut established.closing {
            closing.informed = true;
            established.builder.put(SendFrame {
                priority: u32::MAX,
                time: Instant::now(),
                flags: FrameFlags::IDENTIFIED,
                ftype: FrameType::Control,
                reliable: false,
                order: None,
                ident: Some(ControlFrameIdent::FullyClose.into()),
                payload: Bytes::new(),
            });
        }
    });
}

pub(in crate::connection) fn established_close_despawn_system(
    mut commands: Commands,
    mut connections: Query<(Entity, &Connection, &Established, Option<&mut NetworkPeerLifestage>)>,
    mut endpoints: Query<&mut Endpoint>,
    mut events: EventWriter<PeerDisconnectedEvent>,
) {
    for (peer, connection, established, lifestage) in connections.iter_mut() {
        let reason = match &established.closing {
            Some(r) => {
                if !r.finished { continue }
                r.reason.clone()
            },
            None => { continue },
        };

        // At this point, we know they're finished

        info!("Peer {peer:?} disconnected");

        commands.entity(peer).despawn();

        let endpoint = connection.owning_endpoint;
        match endpoints.get_mut(endpoint) {
            Ok(mut endpoint) => {
                endpoint.remove_peer(peer);
            },
            Err(_) => {
                error!("Tried to remove {peer:?} from {endpoint:?} but it wasn't in the map");
            },
        }

        if let Some(mut lifestage) = lifestage {
            *lifestage = NetworkPeerLifestage::Closed;
        }

        events.send(PeerDisconnectedEvent { peer, reason });
    }
}