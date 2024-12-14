pub(super) fn new() -> (
    OutgoingConnectionRequest,
    OutgoingConnectionReceiver,
) {
    let (tx, rx) = async_channel::bounded(1);

    let request = OutgoingConnectionRequest { tx };
    let reciever = OutgoingConnectionReceiver { rx };

    return (request, reciever);
}

pub(super) struct OutgoingConnectionRequest {
    tx: async_channel::Sender<Response>,
}

impl OutgoingConnectionRequest {
    pub fn accept(self, data: impl Into<AcceptedData>) {
        let _ = self.tx.send_blocking(Response::Accepted(data.into()));
    }

    pub fn reject(self, data: impl Into<RejectedData>) {
        let _ = self.tx.send_blocking(Response::Rejected(data.into()));
    }
}

pub(super) struct OutgoingConnectionReceiver {
    pub rx: async_channel::Receiver<Response>,
}

pub(super) enum Response {
    Accepted(AcceptedData),
    Rejected(RejectedData),
}

pub(super) struct AcceptedData {

}

pub(super) struct RejectedData {

}