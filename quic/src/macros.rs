macro_rules! pin_poll {
    ($val:expr, $cx:ident) => {
        {
            let x = std::pin::pin!($val);
            x.poll($cx)
        }
    };
}

pub(crate) use pin_poll;