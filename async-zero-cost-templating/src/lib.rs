extern crate alloc;

pub use async_zero_cost_templating_proc_macro::html;
use std::convert::Infallible;

use bytes::Bytes;
use futures_core::{Future, Stream};

use http_body::{Body, Frame};

// The reason we use a channel for now it that we want to be able to template values that don't have a lifetime of 'static and it seems like our Cell hack doesn't allow this because of invariance?
// Because we also want to be able to send values with a lifetime of static depening on the use case (all returned values live forever).

// we don't want to use an unstable edition so we can't use `async gen`
// we don't want to use unsafe so we can't use an async coroutine lowering
// RUSTFLAGS="-Zprint-type-sizes" cargo build > target/type-sizes.txt
// `{async fn
// __awaitee is the thing we're currently awaiting

// it should emit blocks of a specified size to reduce fragmentation. This means the goal is not always lowest latency but little overhead and then lowest latency

pub struct TemplateHttpBody<S: Stream<Item = Bytes>> {
    pub stream: S,
    pub chunk_size: usize,
}

impl<S: Stream<Item = Bytes>> Body for TemplateHttpBody<S> {
    type Data = &'static [u8];

    type Error = Infallible;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        todo!()
    }
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/compile_fail/*.rs");
    t.pass("tests/ui/pass/*.rs");
}

pub struct TemplateToStream<F: Future<Output = ()> + Send> {
    future: F
}