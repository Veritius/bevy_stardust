use std::marker::PhantomData;
use crate::prelude::*;

#[derive(Default)]
pub(crate) struct ReplicationData<T: Replicable>(PhantomData<T>);