use std::{future::Future, pin::Pin, task::{Context, Poll}};

pub(crate) struct Join<T>(pub T);

macro_rules! set_try {
    ($x:expr, $cx:ident) => {
        match Pin::new($x).poll($cx) {
            Poll::Ready(v) => return Poll::Ready(v),
            Poll::Pending => {},
        }
    };
}

macro_rules! join_tuple_impl {
    ([$($t:ident:$b:tt),+]) => {
        impl<$($t),+, Z> Future for Join<($($t),+)>
        where $($t: Future<Output = Z> + Unpin),+ {
            type Output = Z;

            fn poll(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Z> {
                let x = &mut self.0;
            
                $(set_try!(&mut x.$b, cx);)+

                return Poll::Pending;
            }
        }
    };
}

macro_rules! variadic_tuple {
    ($p:ident) => {
        $p!([A:0, B:1]);
        $p!([A:0, B:1, C:2]);
        $p!([A:0, B:1, C:2, D:3]);
        $p!([A:0, B:1, C:2, D:3, E:4]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14]);
        $p!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15]);
    };
}

variadic_tuple!(join_tuple_impl);