mod codes;

pub(in crate::connection) use codes::HandshakeResponseCode;

#[derive(Default)]
pub(super) struct HandshakeStateMachine {
    state: HandshakeStateInner,
}

#[derive(Default)]
enum HandshakeStateInner {
    #[default]
    Uninitialised,
    InitiatorHello,
    ListenerResponse,
    InitiatorResponse,
}