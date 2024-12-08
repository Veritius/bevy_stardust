use std::{fmt::{Debug, Display}, sync::atomic::{AtomicUsize, Ordering}};

pub(crate) enum LogIdGen {}

impl LogIdGen {
    pub fn next() -> LogId {
        static INDEX: AtomicUsize = AtomicUsize::new(0);
        let idx = INDEX.fetch_add(1, Ordering::Relaxed);
        return LogId(idx);
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub(crate) struct LogId(usize);

impl Debug for LogId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <usize as Debug>::fmt(&self.0, f)
    }
}

impl Display for LogId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <usize as Display>::fmt(&self.0, f)
    }
}