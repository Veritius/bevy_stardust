use std::{mem::swap, time::Instant};
use bevy::prelude::*;
use bevy_stardust_extras::varint::VarInt;
use bytes::Bytes;
use closing::CloseOrigin;
use frames::frames::{FrameType, SendFrame};
use super::*;

pub(super) struct ControlFrame {
    pub ident: ControlFrameIdent,
    pub payload: Bytes
}

pub(super) enum ControlFrameIdent {
    BeginClose,
    FullyClose,
}

impl TryFrom<VarInt> for ControlFrameIdent {
    type Error = ();

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        use ControlFrameIdent::*;
        let c = <VarInt as Into<u64>>::into(value) as u32;
        Ok(match c {
            0 => BeginClose,
            1 => FullyClose,
            _ => { return Err(()); }
        })
    }
}

impl From<ControlFrameIdent> for VarInt {
    fn from(value: ControlFrameIdent) -> Self {
        use ControlFrameIdent::*;
        let v = match value {
            BeginClose => 0,
            FullyClose => 1,
        };
        
        VarInt::from_u32(v)
    }
}

pub(in crate::connection) fn established_control_system(
    mut connections: Query<&mut Established>,
) {
    connections.par_iter_mut().for_each(|mut established| {
        // Swap the buffer out of the component to drain it
        // We have to do this so we don't hold a borrow on established
        let mut control = SmallVec::new();
        swap(&mut established.control, &mut control);
        let iter = control.drain(..).rev();

        for frame in iter {
            use ControlFrameIdent::*;

            match frame.ident {
                BeginClose => {
                    match &mut established.closing {
                        Some(closing) => {
                            closing.informed = true;
                        },

                        None => {
                            established.closing = Some(Closing {
                                finished: false,
                                informed: true,
                                origin: CloseOrigin::Remote,
                                reason: match frame.payload.len() {
                                    0 => None,
                                    _ => Some(frame.payload),
                                }
                            });
                        },
                    }

                    established.builder.put(SendFrame {
                        priority: u32::MAX,
                        time: Instant::now(),
                        ftype: FrameType::Control,
                        reliable: false,
                        order: None,
                        ident: Some(ControlFrameIdent::FullyClose.into()),
                        payload: Bytes::new(),
                    });
                },

                FullyClose => {
                    match &mut established.closing {
                        Some(closing) => {
                            closing.finished = true;
                        },

                        None => {
                            established.closing = Some(Closing {
                                finished: true,
                                informed: true,
                                origin: CloseOrigin::Remote,
                                reason: None,
                            });
                        }
                    }
                },
            }
        }

        // Swap the buffer back into the component
        swap(&mut established.control, &mut control);
    });
}