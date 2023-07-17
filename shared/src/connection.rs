use crate::channel::Channel;

/// Special channel that is used by the client/server to swap cryptographic information.
#[derive(Debug)]
pub(crate) struct AuthenticationChannel;
impl Channel for AuthenticationChannel {}