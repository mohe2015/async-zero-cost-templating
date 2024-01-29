use core::cell::Cell;
use std::{borrow::Cow, pin::Pin, task::Poll};

use bytes::Bytes;
use futures_core::{Future, Stream};
use pin_project::pin_project;

pub type T = ::alloc::borrow::Cow<'static, str>;

thread_local! {
    static VALUE: Cell<Option<T>> = const { Cell::new(None) };
}

// Should not be publicly constructable
#[derive(Copy, Clone)]
pub struct FutureToStream(());

impl FutureToStream {
    pub fn _yield(self, value: T) -> FutureToStream {
        VALUE.set(Some(value));
        self
    }
}

impl Future for FutureToStream {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match VALUE.take() {
            Some(value) => {
                VALUE.set(Some(value));
                Poll::Pending
            }
            None => Poll::Ready(()),
        }
    }
}

#[pin_project]
pub struct TheStream<F: Future<Output = ()>> {
    #[pin]
    future: F,
}

impl<F: Future<Output = ()>> TheStream<F> {
    pub fn new(input: impl FnOnce(FutureToStream) -> F) -> Self {
        Self {
            future: input(FutureToStream(())),
        }
    }
}

impl<F: Future<Output = ()>> Stream for TheStream<F> {
    type Item = T;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let result = this.future.poll(cx);
        match result {
            Poll::Ready(_) => Poll::Ready(None),
            Poll::Pending => {
                if let Some(value) = VALUE.take() {
                    Poll::Ready(Some(value))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
