use std::{future::Future, pin::Pin, task::{Context, Poll}};

// pub struct Select<S: Selectable>(pub S);

// impl<S> Future for Select<S> {
//     type Output = ();

//     fn poll(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//     ) -> Poll<Self::Output> {
//         todo!()
//     }
// }

trait Selectable
where
    Self: Unpin,
{
    fn select_poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<()>;
}

impl<T> Selectable for T
where
    T: Unpin,
    T: Future<Output = ()>,
{
    #[inline]
    fn select_poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<()> {
        self.poll(cx)
    }
}

macro_rules! set_try {
    ($x:expr, $cx:ident) => {
        match Pin::new($x).select_poll($cx) {
            Poll::Ready(_) => return Poll::Ready(()),
            Poll::Pending => {},
        }
    };
}

struct Set<T>(T);

macro_rules! set_impl {
    ([$($t:ident:$b:tt),+]) => {
        impl<$($t),+> Selectable for Set<($($t),+)>
        where $($t: Selectable),+ {
            fn select_poll(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<()> {
                let x = &mut self.0;
            
                $(set_try!(&mut x.$b, cx);)+

                return Poll::Pending;
            }
        }
    };
}

set_impl!([A:0, B:1]);
set_impl!([A:0, B:1, C:2]);
set_impl!([A:0, B:1, C:2, D:3]);
set_impl!([A:0, B:1, C:2, D:3, E:4]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14]);
set_impl!([A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15]);