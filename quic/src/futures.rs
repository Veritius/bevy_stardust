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

join_tuple_impl!([A:0, B:1]);
join_tuple_impl!([A:0, B:1, C:2]);
join_tuple_impl!([A:0, B:1, C:2, D:3]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14]);
join_tuple_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15]);