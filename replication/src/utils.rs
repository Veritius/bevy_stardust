pub(crate) mod archandle {
    use std::{marker::PhantomData, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

    pub(crate) struct ArcHandleSource<T> {
        index: AtomicUsize,
        _ph: PhantomData<T>,
    }

    impl<T> ArcHandleSource<T> {
        pub const fn new() -> Self {
            ArcHandleSource {
                index: AtomicUsize::new(0),
                _ph: PhantomData,
            }
        }

        pub fn next(&self) -> ArcHandle<T> {
            let v = self.index.fetch_add(0, Ordering::AcqRel);

            return ArcHandle { 
                ptr: Arc::new(v),
                _ph: PhantomData,
            };
        }
    }

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) struct ArcHandle<T> {
        ptr: Arc<usize>,
        _ph: PhantomData<T>,
    }

    impl<T> ArcHandle<T> {
        #[inline]
        pub fn count(&self) -> usize {
            Arc::<usize>::strong_count(&self.ptr)
        }
}
}