use std::{collections::VecDeque, sync::{Arc, Mutex, MutexGuard}};

pub(crate) fn event_pair() -> (ConnectionEventSender, ConnectionEventReceiver) {
    let queue = EventQueue(Arc::new(Mutex::new(EventQueueInner {
        queue: VecDeque::new(),
    })));

    return (
        ConnectionEventSender(queue.clone()),
        ConnectionEventReceiver(queue),
    )
}

pub(crate) struct ConnectionEventSender(EventQueue);

impl ConnectionEventSender {
    pub fn lock(&self) -> EventSenderLock {
        EventSenderLock(self.0.lock())
    }
}


pub(crate) struct EventSenderLock<'a>(EventQueueLock<'a>);

impl EventSenderLock<'_> {
    pub fn push(&mut self, event: ConnectionEvent) {
        self.0.0.push(event)
    }
}

pub(crate) struct ConnectionEventReceiver(EventQueue);

impl ConnectionEventReceiver {
    pub fn lock(&self) -> EventReceiverLock {
        EventReceiverLock(self.0.lock())
    }
}

pub(crate) struct EventReceiverLock<'a>(EventQueueLock<'a>);

impl EventSenderLock<'_> {
    pub fn pop(&mut self) -> Option<ConnectionEvent> {
        self.0.0.pop()
    }
}

#[derive(Clone)]
struct EventQueue(Arc<Mutex<EventQueueInner>>);

impl EventQueue {
    fn lock(&self) -> EventQueueLock {
        EventQueueLock(self.0.lock().unwrap())
    }
}

struct EventQueueLock<'a>(MutexGuard<'a, EventQueueInner>);

struct EventQueueInner {
    queue: VecDeque<ConnectionEvent>,
}

impl EventQueueInner {
    fn push(&mut self, event: ConnectionEvent) {
        self.queue.push_back(event);
    }

    fn pop(&mut self) -> Option<ConnectionEvent> {
        return self.queue.pop_front();
    }

    fn clear(&mut self) {
        self.queue.clear();
    }
}

pub(crate) enum ConnectionEvent {

}