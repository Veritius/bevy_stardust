use std::{collections::VecDeque, ops::{Deref, DerefMut}, sync::{Arc, Mutex, MutexGuard}};

pub(crate) fn event_pair<T>() -> (EventSender<T>, EventReader<T>) {
    let events = Events {
        queue: VecDeque::new(),
    };

    let arc = EventsArc(
        Arc::new(Mutex::new(events))
    );

    return (
        EventSender(arc.clone()),
        EventReader(arc.clone()),
    )
}

pub(crate) struct EventSender<T>(EventsArc<T>);

impl<T> EventSender<T> {
    pub fn lock(&self) -> EventSenderLock<'_, T> {
        EventSenderLock(self.0.lock())
    }
}

pub(crate) struct EventSenderLock<'a, T>(EventsLock<'a, T>);

impl<T> EventSenderLock<'_, T> {
    pub fn push(&mut self, event: T) {
        self.0.push(event)
    }
}

pub(crate) struct EventReader<T>(EventsArc<T>);

impl<T> EventReader<T> {
    pub fn lock(&self) -> EventReaderLock<'_, T> {
        EventReaderLock(self.0.lock())
    }
}

pub(crate) struct EventReaderLock<'a, T>(EventsLock<'a, T>);

impl<T> EventReaderLock<'_, T> {
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

struct EventsArc<T>(Arc<Mutex<Events<T>>>);

impl<T> EventsArc<T> {
    fn lock(&self) -> EventsLock<'_, T> {
        EventsLock { lock: self.0.lock().unwrap() }
    }
}

impl<T> Clone for EventsArc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct Events<T> {
    queue: VecDeque<T>,
}

impl<T> Events<T> {
    fn push(&mut self, event: T) {
        self.queue.push_back(event)
    }

    fn pop(&mut self) -> Option<T> {
        self.queue.pop_front()
    }
}

struct EventsLock<'a, T> {
    lock: MutexGuard<'a, Events<T>>,
}

impl<T> Deref for EventsLock<'_, T> {
    type Target = Events<T>;

    fn deref(&self) -> &Self::Target {
        &self.lock
    }
}

impl<T> DerefMut for EventsLock<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lock
    }
}