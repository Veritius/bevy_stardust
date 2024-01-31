mod incoming;
mod outgoing;
use crate::connection::inner::Connection;

pub(crate) enum AttemptOutcome {
    Unfinished,
    Success(Connection),
    Failure(HandshakeFailure),
}

pub(crate) enum HandshakeFailure {
    
}