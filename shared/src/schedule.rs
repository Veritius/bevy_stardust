use bevy::ecs::schedule::ScheduleLabel;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum NetworkReceive {
    ReadMessages,
    SendTypes,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum NetworkTransmit {
    Transmit,
    ClearBuffers,
}