use std::{future::Future, pin::Pin};

pub trait Runtime
where
    Self: Send + Sync + 'static,
{
    fn spawn(
        &self,
        fut: Pin<Box<dyn Future<Output = ()> + Send>>,
    );
}