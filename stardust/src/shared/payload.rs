use super::octetstring::OctetString;

/// All [Payload] objects for a channel. If the channel this originated from is ordered, the [Payload]s will be in order.
pub struct Payloads(pub Box<[Payload]>);

impl From<Box<[Payload]>> for Payloads {
    fn from(value: Box<[Payload]>) -> Self {
        Self(value)
    }
}

impl From<Vec<Payload>> for Payloads {
    fn from(value: Vec<Payload>) -> Self {
        Self(value.into_boxed_slice())
    }
}

/// A single network message sent over a channel, free of any additional transmission information when read.
pub struct Payload {
    pub(crate) ignore_head: usize,
    pub(crate) ignore_tail: usize,
    pub(crate) data: OctetString,
}

impl Payload {
    pub fn new(head: usize, tail: usize, data: impl Into<OctetString>) -> Self {
        Self {
            ignore_head: head,
            ignore_tail: tail,
            data: data.into(),
        }
    }

    /// Hides a certain amount of bytes from the head and tail of the octet string.
    pub fn hide(&mut self, head: usize, tail: usize) {
        self.ignore_head += head;
        self.ignore_tail += tail;
    }

    /// Gives access to the relevant octets of the message.
    pub fn read(&self) -> &[u8] {
        let data = self.data.as_slice();
        let len = data.len();
        &data[self.ignore_head-1..(len - self.ignore_tail)]
    }
}