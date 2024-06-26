//! A simple transport layer using inter-thread communications, intended for use in tests and examples.
//! 
//! Usage is simple, just add [`LinkTransportPlugin`] to all involved apps.
//! Then, use [`pair`] to create two [`Link`] components that communicate with eachother.
//! These 'links' don't do any kind of handshake. Once added to an entity, they communicate immediately.

use std::sync::{mpsc::{channel, Receiver, Sender, TryRecvError}, Mutex};
use bevy::prelude::*;
use crate::prelude::*;

/// Adds a simple transport plugin for apps part of the same process.
/// See the [top level documentation](self) for more information.
pub struct LinkTransportPlugin;

impl Plugin for LinkTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (recv_link_data, remove_disconnected)
            .chain().in_set(NetworkRecv::Receive));

        app.add_systems(PostUpdate, (send_link_data, remove_disconnected)
            .chain().in_set(NetworkSend::Transmit));
    }
}

/// A connection to another `Link`, made with [`pair`].
/// 
/// A `Link` will only communicate with its counterpart.
#[derive(Component)]
pub struct Link(SideInner);

/// Creates two connected [`Link`] objects.
pub fn pair() -> (Link, Link) {
    let (left_tx, left_rx) = channel();
    let (right_tx, right_rx) = channel();

    let left = Link(SideInner {
        sender: left_tx,
        receiver: Mutex::new(right_rx),
        disconnected: false,
    });

    let right = Link(SideInner {
        sender: right_tx,
        receiver: Mutex::new(left_rx),
        disconnected: false,
    });

    return (left, right);
}

struct SideInner {
    sender: Sender<ChannelMessage>,
    // Makes the struct Sync, so it can be in a Component.
    // Use Exclusive when it's stabilised.
    receiver: Mutex<Receiver<ChannelMessage>>,
    disconnected: bool,
}

fn recv_link_data(
    mut query: Query<(&mut Link, &mut NetworkMessages<Incoming>), With<NetworkPeer>>,
) {
    query.par_iter_mut().for_each(|(mut link, mut queue)| {
        let receiver = link.0.receiver.get_mut().unwrap();
        loop {
            match receiver.try_recv() {
                Ok(message) => {
                    queue.push(message.channel, message.payload);
                },
                Err(TryRecvError::Empty) => { break },
                Err(TryRecvError::Disconnected) => {
                    link.0.disconnected = true;
                    break;
                },
            }
        }
    });
}

fn send_link_data(
    mut query: Query<(&mut Link, &NetworkMessages<Outgoing>), With<NetworkPeer>>,
) {
    query.par_iter_mut().for_each(|(mut link, queue)| {
        let sender = &link.0.sender;
        'outer: for (channel, queue) in queue.iter() {
            for payload in queue {
                match sender.send(ChannelMessage { channel, payload }) {
                    Ok(_) => {},
                    Err(_) => {
                        link.0.disconnected = true;
                        break 'outer;
                    },
                }
            }
        }
    });
}

fn remove_disconnected(
    mut commands: Commands,
    mut query: Query<(Entity, &Link, Option<&mut NetworkPeerLifestage>)>,
) {
    for (entity, link, stage) in query.iter_mut() {
        if link.0.disconnected {
            debug!("Link on entity {entity:?} disconnected");
            commands.entity(entity).remove::<Link>();
            if let Some(mut stage) = stage {
                *stage = NetworkPeerLifestage::Closed;
            }
        }
    }
}