use bevy::ecs::schedule::ScheduleLabel;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum NetworkReceiveLabels {
    Receive,
    Process,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum NetworkTransmitLabels {
    Process,
    Transmit,
    ClearBuffer,
}