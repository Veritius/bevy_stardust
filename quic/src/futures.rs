use std::{future::Future, marker::PhantomData, pin::Pin, task::{Context, Poll}};

pub(crate) struct Select<E, S> {
    pub context: S,
    pub entries: E,
}

pub(crate) struct SelectEntry<S, F, C, Z>
where
    F: Future<Output = C>,
    C: FnMut(S) -> Z,
{
    future: F,

    _p1: PhantomData<S>,
}

macro_rules! select_tuple_impl {
    ([$($t:ident:$b:tt),+]) => {
        paste::paste! {
            impl<S, $([<$t F>], [<$t C>]),+, Z> Future for Select<($(SelectEntry<S, [<$t F>], [<$t C>], [<Z>]>),+), S>
            where
                S: Unpin + Copy,
                $(
                    [<$t F>]: Future<Output = [<$t C>]> + Unpin,
                    [<$t C>]: FnMut(S) -> Z
                ),+
            {
                type Output = Z;
            
                fn poll(
                    mut self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                ) -> Poll<Z> {
                    let e = &mut self.entries;

                    $(match Pin::new(&mut e.$b.future).poll(cx) {
                        Poll::Ready(mut v) => return Poll::Ready(v(self.context)),
                        Poll::Pending => {},
                    })+

                    return Poll::Pending;
                }
            }
        }
    };
}

pub(crate) struct Join<T>(pub T);

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

                $(match Pin::new(&mut x.$b).poll(cx) {
                    Poll::Ready(v) => return Poll::Ready(v),
                    Poll::Pending => {},
                })+

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

variadic_tuple!(select_tuple_impl);
variadic_tuple!(join_tuple_impl);