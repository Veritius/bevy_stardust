use bevy::{prelude::*, reflect::Reflect};
use crate::setup::MultiplayerMode;

/// What state the game is in, networking-wise.
/// 
/// Some states will not occur, based on the `MultiplayerMode`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, States)]
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

/// Checks for invalid state changes and state values.
pub(crate) fn state_machine_checker(
    mode: Res<MultiplayerMode>,
    mut last_state: Local<Option<MultiplayerState>>,
    state: Res<State<MultiplayerState>>,
) {
    // The mode should never change
    if mode.is_changed() && !mode.is_added() { panic!("The MultiplayerMode enum was changed. This should never happen!"); }

    // Nothing's changed, move on
    if !state.is_changed() { return; }

    if last_state.is_none() {
        // Update last_state. This only happens once.
        *last_state = Some(state.get().clone());
    }

    match (*mode, state.get()) {
        (MultiplayerMode::DedicatedServer, MultiplayerState::Disconnected) |
        (MultiplayerMode::DedicatedServer, MultiplayerState::Singleplayer) |
        (MultiplayerMode::DedicatedServer, MultiplayerState::JoiningRemote) |
        (MultiplayerMode::DedicatedServer, MultiplayerState::JoinedRemote) |
        (MultiplayerMode::DedicatedClient, MultiplayerState::Singleplayer) |
        (MultiplayerMode::DedicatedClient, MultiplayerState::StartingServer) |
        (MultiplayerMode::DedicatedClient, MultiplayerState::RunningServer) |
        (MultiplayerMode::ClientAndHost, MultiplayerState::Singleplayer) |
        (MultiplayerMode::ClientWithSingleplayer, MultiplayerState::StartingServer) |
        (MultiplayerMode::ClientWithSingleplayer, MultiplayerState::RunningServer) =>
            panic!("Invalid MultiplayerState: {:?} can never be {:?}", *mode, state.get()),
        _ => {}
    }
}

/// Conditional for systems. Only returns `true` if this app is currently hosting or connected to a remote server.
pub fn is_connected() -> impl Fn(Res<'_, State<MultiplayerState>>) -> bool + Clone {
    move |state: Res<State<MultiplayerState>>| {
        match state.get() {
            MultiplayerState::RunningServer => true,
            MultiplayerState::JoinedRemote => true,
            _ => false,
        }
    }
}

/// Conditional for systems. Only returns `true` if this app is currently acting as a client.
pub fn is_client() -> impl Fn(Res<'_, State<MultiplayerState>>) -> bool + Clone {
    move |state: Res<State<MultiplayerState>>| {
        match state.get() {
            MultiplayerState::JoinedRemote => true,
            _ => false,
        }
    }
}

/// Conditional for systems. Only returns `true` if this app is currently acting as a server.
pub fn is_server() -> impl Fn(Res<'_, State<MultiplayerState>>) -> bool + Clone {
    move |state: Res<State<MultiplayerState>>| {
        match state.get() {
            MultiplayerState::RunningServer => true,
            _ => false,
        }
    }
}