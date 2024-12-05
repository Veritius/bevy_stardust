pub mod mpmc {
    use std::fmt::Debug;

    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = crossbeam_channel::unbounded::<T>();
        return (Sender(tx), Receiver(rx));
    }

    pub(crate) struct Sender<T>(crossbeam_channel::Sender<T>);

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Sender<T> {
        pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
            self.0.send(msg).map_err(|v| SendError(v.0))
        }
    }

    pub(crate) struct Receiver<T>(crossbeam_channel::Receiver<T>);

    impl<T> Clone for Receiver<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Receiver<T> {
        pub fn try_recv(&self) -> Result<T, TryRecvError> {
            self.0.try_recv().map_err(|v| match v {
                crossbeam_channel::TryRecvError::Empty => TryRecvError::Empty,
                crossbeam_channel::TryRecvError::Disconnected => TryRecvError::Disconnected,
            })
        }
    }

    pub(crate) struct SendError<T>(pub T);

    impl<T> Debug for SendError<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("SendError")
        }
    }

    #[derive(Debug)]
    pub(crate) enum TryRecvError {
        Empty,
        Disconnected,
    }
}

pub mod mpsc {
    use std::fmt::Debug;

    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = crossbeam_channel::unbounded::<T>();
        return (Sender(tx), Receiver(rx));
    }

    pub(crate) struct Sender<T>(crossbeam_channel::Sender<T>);

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Sender<T> {
        pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
            self.0.send(msg).map_err(|v| SendError(v.0))
        }
    }

    pub(crate) struct Receiver<T>(crossbeam_channel::Receiver<T>);

    impl<T> Receiver<T> {
        pub fn try_recv(&self) -> Result<T, TryRecvError> {
            self.0.try_recv().map_err(|v| match v {
                crossbeam_channel::TryRecvError::Empty => TryRecvError::Empty,
                crossbeam_channel::TryRecvError::Disconnected => TryRecvError::Disconnected,
            })
        }
    }

    pub(crate) struct SendError<T>(pub T);

    impl<T> Debug for SendError<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("SendError")
        }
    }

    #[derive(Debug)]
    pub(crate) enum TryRecvError {
        Empty,
        Disconnected,
    }
}

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
    use std::{future::Future, task::Poll};

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
        pub fn send(self, value: T) -> Result<(), SendError<T>> {
            self.inner.send(value).map_err(|v| SendError(v.0))
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

        pub fn inner(&self) -> &crossbeam_channel::Receiver<T> {
            &self.inner
        }
    }

    impl<T> Future for Receiver<T> {
        type Output = Option<T>;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            match self.inner.try_recv() {
                Ok(event) => return Poll::Ready(Some(event)),
                Err(crossbeam_channel::TryRecvError::Empty) => Poll::Pending,
                Err(crossbeam_channel::TryRecvError::Disconnected) => Poll::Ready(None),
            }
        }
    }

    #[derive(Debug)]
    pub struct SendError<T>(pub T);

    #[derive(Debug)]
    pub(crate) enum TryRecvError {
        Empty,
        Disconnected,
    }
}