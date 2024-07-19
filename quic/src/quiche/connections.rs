use crate::connection::ConnectionBackend;

pub struct QuicheConnection {

}

impl ConnectionBackend for QuicheConnection {
    fn is_closed(&self) -> bool {
        todo!()
    }
}