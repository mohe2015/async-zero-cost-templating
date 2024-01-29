use core::cell::Cell;
use std::{pin::Pin, task::Poll};

use futures_core::{Future, Stream};
use pin_project::pin_project;

// we need to store the future in the stream thingy because we need to poll it
// we need to be able to pass down a reference of the value to write because of nested stuff (maybe a FutureToStreamRef)
// this doesn't need to be perfectly beautiful because we only use it in the codegen

pub struct FutureToStream<T>(pub Cell<Option<T>>);

impl<T> FutureToStream<T> {
    pub fn _yield(&self, value: T) -> &Self {
        self.0.set(Some(value));
        self
    }
}

impl<T> Future for &FutureToStream<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.0.take() {
            Some(value) => {
                self.0.set(Some(value));
                Poll::Pending
            }
            None => Poll::Ready(()),
        }
    }
}

#[pin_project]
pub struct TheStream<'a, T, F: Future<Output = ()>> {
    future_to_stream: &'a FutureToStream<T>,
    #[pin]
    future: F,
}

impl<'a, T, F: Future<Output = ()>> TheStream<'a, T, F> {
    pub fn new(future_to_stream: &'a FutureToStream<T>, future: F) -> Self {
        Self {
            future_to_stream,
            future,
        }
    }
}

impl<'a, T, F: Future<Output = ()>> Stream for TheStream<'a, T, F> {
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
                if let Some(value) = this.future_to_stream.0.take() {
                    Poll::Ready(Some(value))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
