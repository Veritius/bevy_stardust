use bevy::{prelude::*, reflect::Reflect};

/// What state the game is in, networking-wise.
/// 
/// Some states will not occur, based on the `MultiplayerMode`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, States)]
#[non_exhaustive]
pub enum MultiplayerState {
    /// Not currently hosting or connected to anything.
    #[default]
    Disconnected,
    /// Currently in singleplayer.
    Singleplayer,

    /// Starting a server.
    StartingServer,
    /// Running a server.
    RunningServer,

    /// Trying to join a server.
    JoiningRemote,
    /// Connected to a server.
    JoinedRemote,
}

impl MultiplayerState {
    pub fn in_singleplayer(&self) -> bool {
        match self {
            MultiplayerState::Singleplayer => true,
            _ => false,
        }
    }

    pub fn in_multiplayer(&self) -> bool {
        match self {
            Self::Disconnected | Self::Singleplayer => false,
            _ => true,
        }
    }

    pub fn is_server(&self) -> bool {
        match self {
            Self::StartingServer | Self::RunningServer => true,
            _ => false,
        }
    }

    pub fn is_client(&self) -> bool {
        match self {
            Self::JoiningRemote | Self::JoinedRemote => true,
            _ => false,
        }
    }
}