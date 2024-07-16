use anyhow::Result;
use crate::{endpoint::{ClientReady, ServerReady, DualReady}, Endpoint};

pub(crate) fn build_client(state: ClientReady) -> Result<Endpoint> {
    todo!()
}

pub(crate) fn build_server(state: ServerReady) -> Result<Endpoint> {
    todo!()
}

pub(crate) fn build_dual(state: DualReady) -> Result<Endpoint> {
    todo!()
}