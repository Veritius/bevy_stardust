use std::marker::PhantomData;
use crate::prelude::*;

#[derive(Default)]
pub(crate) struct ResourceReplicationData<T: ReplicableResource>(PhantomData<T>);

#[derive(Default)]
pub(crate) struct ComponentReplicationData<T: ReplicableComponent>(PhantomData<T>);

#[derive(Default)]
pub(crate) struct EventReplicationData<T: ReplicableEvent>(PhantomData<T>);