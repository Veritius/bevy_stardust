use bevy::ecs::schedule::ScheduleLabel;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum NetworkReceive {
    Receive,
    Process,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum NetworkTransmit {
    Process,
    Transmit,
    ClearBuffer,
}