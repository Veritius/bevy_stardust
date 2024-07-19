use crate::connection::ConnectionState;

pub struct QuicheConnection {
    connection: quiche::Connection,
}

impl ConnectionState for QuicheConnection {
    type Backend = super::Quiche;

    fn is_closed(&self) -> bool {
        todo!()
    }
}