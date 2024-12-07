use std::{future::Future, marker::PhantomData, pin::Pin, task::{Context, Poll}};

pub(crate) struct Race<Output, Set> {
    futures: Set,
    phantom: PhantomData<Output>,
}

impl<Output, Set> Race<Output, Set> {
    pub fn new(futures: Set) -> Self {
        Self {
            futures,
            phantom: PhantomData,
        }
    }
}

macro_rules! impl_race_tuple {
    ([$($t:ident:$n:tt),+]) => {
        impl<Output, $($t),+> Future for Race<Output, ($($t),+)>
        where
            Output: Unpin,
            $($t: Future<Output = Output> + Unpin),+
        {
            type Output = Output;

            fn poll(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Self::Output> {
                let e = &mut self.futures;

                $(
                    if let Poll::Ready(value) = Pin::new(&mut e.$n).poll(cx) {
                        return Poll::Ready(value);
                    }
                )+

                return Poll::Pending;
            }
        }
    };
}

macro_rules! impl_variadics {
    ($m:ident) => {
        $m!([A:0, B:1]);
        $m!([A:0, B:1, C:2 ]);
        $m!([A:0, B:1, C:2, D:3 ]);
        $m!([A:0, B:1, C:2, D:3, E:4 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8, K:9 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8, K:9, L:10 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8, K:9, L:10, M:11 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8, K:9, L:10, M:11, N:12 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8, K:9, L:10, M:11, N:12, O:13 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8, K:9, L:10, M:11, N:12, O:13, P:14 ]);
        $m!([A:0, B:1, C:2, D:3, E:4, F:5, H:6, I:7, J:8, K:9, L:10, M:11, N:12, O:13, P:14, Q:15 ]);
    };
}

impl_variadics!(impl_race_tuple);