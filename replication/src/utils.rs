use std::{cmp::Ordering, sync::Arc};

#[derive(Clone)]
pub(crate) struct ArcHandle(Arc<()>);

impl ArcHandle {
    /// Creates a new unique [`ArcHandle`].
    pub fn new() -> Self {
        ArcHandle(Arc::new(()))
    }

    #[inline]
    pub fn count(&self) -> usize {
        Arc::<()>::strong_count(&self.0)
    }

    #[inline]
    fn addr(&self) -> *const () {
        Arc::<()>::as_ptr(&self.0)
    }
}

impl PartialEq for ArcHandle {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            self.addr(),
            other.addr()
        )
    }
}

impl Eq for ArcHandle {}

impl PartialOrd for ArcHandle {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.addr().partial_cmp(&other.addr())
    }
}

impl Ord for ArcHandle {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr().cmp(&other.addr())
    }
}