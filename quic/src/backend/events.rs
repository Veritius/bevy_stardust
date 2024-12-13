pub(crate) enum E2CEvent {
    EndpointClosed,
    Quinn(quinn_proto::ConnectionEvent),
}

impl From<quinn_proto::ConnectionEvent> for E2CEvent {
    fn from(value: quinn_proto::ConnectionEvent) -> Self {
        Self::Quinn(value)
    }
}

pub(crate) enum C2EEvent {
    ConnectionClosed,
    Quinn(quinn_proto::EndpointEvent),
}

impl From<quinn_proto::EndpointEvent> for C2EEvent {
    fn from(value: quinn_proto::EndpointEvent) -> Self {
        Self::Quinn(value)
    }
}