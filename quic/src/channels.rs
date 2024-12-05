pub mod watch {
    use std::sync::{Arc, Mutex, MutexGuard};

    pub(crate) fn channel<T>(initial: T) -> (Sender<T>, Receiver<T>) {
        let inner = Arc::new(Mutex::new(initial));

        let sender = Sender { inner: inner.clone() };
        let receiver = Receiver { inner: inner.clone() };
        return (sender, receiver);
    }

    pub(crate) struct Sender<T> {
        inner: Arc<Mutex<T>>,
    }

    impl<T> Sender<T> {
        pub fn send(&self, state: T) {
            let mut lock = self.inner.lock().unwrap();
            *lock = state;
        }
    }

    #[derive(Clone)]
    pub(crate) struct Receiver<T> {
        inner: Arc<Mutex<T>>
    }

    impl<T> Receiver<T> {
        pub fn borrow(&self) -> Ref<'_, T> {
            return Ref {
                inner: self.inner.lock().unwrap(),
            }
        }
    }

    pub(crate) struct Ref<'a, T> {
        inner: MutexGuard<'a, T>,
    }
}

pub mod oneshot {
    use std::sync::{Arc, Mutex};

    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let inner = Arc::new(Mutex::new(Inner {
            value: None,
        }));

        let sender = Sender { inner: inner.clone() };
        let receiver = Receiver { inner: inner.clone() };
        return (sender, receiver);
    }

    pub(crate) struct Sender<T> {
        inner: Arc<Mutex<Inner<T>>>,
    }

    impl<T> Sender<T> {
        pub fn send(self, value: T) {
            // If the reference count is 1, the other side has been dropped, and we do nothing
            if Arc::strong_count(&self.inner) == 1 { return }

            // Lock mutex and assign value
            let mut lock = self.inner.lock().unwrap();
            lock.value = Some(value);
        }
    }

    pub(crate) struct Receiver<T> {
        inner: Arc<Mutex<Inner<T>>>
    }

    impl<T> Receiver<T> {
        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            // If the reference count is 1, the other side has been dropped, and we return an error.
            if Arc::strong_count(&self.inner) == 1 { return Err(TryRecvError::Disconnected); }

            // Lock the mutex to access the inner value
            let mut lock = self.inner.lock().unwrap();
            if lock.value.is_none() { return Err(TryRecvError::Empty); }

            // Retrieve value from mutex
            let mut value = None;
            std::mem::swap(&mut value, &mut lock.value);
            return Ok(value.unwrap());
        }
    }

    struct Inner<T> {
        value: Option<T>,
    }

    pub(crate) enum TryRecvError {
        Empty,
        Disconnected,
    }
}