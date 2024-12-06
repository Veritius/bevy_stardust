pub mod mpmc {
    use crate::channels::shared::*;

    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = async_channel::unbounded::<T>();
        return (Sender(tx), Receiver(rx));
    }

    pub(crate) struct Sender<T>(async_channel::Sender<T>);

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Sender<T> {
        pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
            self.0.send_blocking(msg).map_err(|v| SendError(v.0))
        }
    }

    impl<T> Unpin for Sender<T> {}

    pub(crate) struct Receiver<T>(async_channel::Receiver<T>);

    impl<T> Clone for Receiver<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Receiver<T> {
        pub async fn recv(&self) -> Result<T, RecvError> {
            self.0.recv().await.map_err(|_| RecvError)
        }

        pub fn try_recv(&self) -> Result<T, TryRecvError> {
            self.0.try_recv().map_err(|v| match v {
                async_channel::TryRecvError::Empty => TryRecvError::Empty,
                async_channel::TryRecvError::Closed => TryRecvError::Closed,
            })
        }
    }

    impl<T> Unpin for Receiver<T> {}
}

pub mod mpsc {
    use crate::channels::shared::*;

    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = async_channel::unbounded::<T>();
        return (Sender(tx), Receiver(rx));
    }

    pub(crate) struct Sender<T>(async_channel::Sender<T>);

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Sender<T> {
        pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
            self.0.send_blocking(msg).map_err(|v| SendError(v.0))
        }
    }

    impl<T> Unpin for Sender<T> {}

    pub(crate) struct Receiver<T>(async_channel::Receiver<T>);

    impl<T> Receiver<T> {
        pub async fn recv(&self) -> Result<T, RecvError> {
            self.0.recv().await.map_err(|_| RecvError)
        }

        pub fn try_recv(&self) -> Result<T, TryRecvError> {
            self.0.try_recv().map_err(|v| match v {
                async_channel::TryRecvError::Empty => TryRecvError::Empty,
                async_channel::TryRecvError::Closed => TryRecvError::Closed,
            })
        }
    }

    impl<T> Unpin for Receiver<T> {}
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

    impl<T> Unpin for Sender<T> {}

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

    impl<T> Unpin for Receiver<T> {}

    pub(crate) struct Ref<'a, T> {
        inner: MutexGuard<'a, T>,
    }
}

pub mod oneshot {
    use std::{future::Future, task::Poll};
    use crate::channels::shared::*;

    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = async_channel::bounded(1);
        let sender = Sender { inner: tx };
        let receiver = Receiver { inner: rx };
        return (sender, receiver);
    }

    pub(crate) struct Sender<T> {
        inner: async_channel::Sender<T>,
    }

    impl<T> Sender<T> {
        pub fn send(self, value: T) -> Result<(), SendError<T>> {
            self.inner.send_blocking(value).map_err(|v| SendError(v.0))
        }
    }

    impl<T> Unpin for Sender<T> {}

    pub(crate) struct Receiver<T> {
        inner: async_channel::Receiver<T>,
    }

    impl<T> Receiver<T> {
        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            match self.inner.try_recv() {
                Ok(v) => Ok(v),

                Err(e) => Err(match e {
                    async_channel::TryRecvError::Empty => TryRecvError::Empty,
                    async_channel::TryRecvError::Closed => TryRecvError::Closed,
                }),
            }
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
                Err(async_channel::TryRecvError::Empty) => Poll::Pending,
                Err(async_channel::TryRecvError::Closed) => Poll::Ready(None),
            }
        }
    }

    impl<T> Unpin for Receiver<T> {}
}

pub mod shared {
    use std::fmt::Debug;

    pub(crate) struct SendError<T>(pub T);

    impl<T> Debug for SendError<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("SendError")
        }
    }

    pub(crate) struct RecvError;

    impl Debug for RecvError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("RecvError")
        }
    }

    #[derive(Debug)]
    pub(crate) enum TryRecvError {
        Empty,
        Closed,
    }
}