use super::octetstring::OctetString;

/// All [Payload] for a channel. If the channel this originated from is ordered, the [Payload]s will be in order.
pub struct Payloads(pub(crate) Box<[Payload]>);

/// A single network message sent over a channel, free of any additional transmission information when read.
pub struct Payload {
    pub(crate) ignore_head: usize,
    pub(crate) ignore_tail: usize,
    pub(crate) data: OctetString,
}

impl Payload {
    /// Gives access to the relevant octets of the message.
    pub fn read(&self) -> &[u8] {
        let data = self.data.as_slice();
        let len = data.len();
        &data[self.ignore_head-1..len - self.ignore_tail]
    }
}