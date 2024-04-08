use crate::Replicable;

#[derive(Default)]
#[cfg_attr(feature="reflect", derive(bevy::reflect::Reflect))]
#[cfg_attr(feature="reflect", reflect(from_reflect = false))]
pub(crate) struct ReplicationData<T: Replicable>(T);