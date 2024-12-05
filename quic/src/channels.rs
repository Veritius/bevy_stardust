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
    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let sender = Sender { inner: tx };
        let receiver = Receiver { inner: rx };
        return (sender, receiver);
    }

    pub(crate) struct Sender<T> {
        inner: crossbeam_channel::Sender<T>,
    }

    impl<T> Sender<T> {
        pub fn send(self, value: T) {
            self.inner.send(value);
        }
    }

    pub(crate) struct Receiver<T> {
        inner: crossbeam_channel::Receiver<T>,
    }

    impl<T> Receiver<T> {
        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            match self.inner.try_recv() {
                Ok(v) => Ok(v),

                Err(e) => Err(match e {
                    crossbeam_channel::TryRecvError::Empty => TryRecvError::Empty,
                    crossbeam_channel::TryRecvError::Disconnected => TryRecvError::Disconnected,
                }),
            }
        }
    }

    pub(crate) enum TryRecvError {
        Empty,
        Disconnected,
    }
}