pub mod future_to_stream;

use std::{
    cell::Cell,
    convert::Infallible,
    future::Future,
    pin::{pin, Pin},
    task::Poll,
};

use bytes::Bytes;
use future_to_stream::{FutureToStream, TheStream};
use futures_core::Stream;
use futures_util::StreamExt as _;
use http_body::{Body, Frame};
use pin_project::pin_project;

// we don't want to use an unstable edition so we can't use `async gen`
// we don't want to use unsafe so we can't use an async coroutine lowering
// RUSTFLAGS="-Zprint-type-sizes" cargo build > target/type-sizes.txt
// `{async fn
// __awaitee is the thing we're currently awaiting

pub async fn stream_example(stream: FutureToStream) {
    stream._yield(1).await;
    stream._yield(2).await;
    stream._yield(3).await;
}

#[tokio::test]
pub async fn test1() {
    let stream = TheStream::new(stream_example);
    let mut stream = pin!(stream);
    while let Some(value) = stream.next().await {
        eprintln!("got {}", value)
    }
    eprintln!("done")
}

macro_rules! html {
    ($($tt: tt)*) => {};
}

// it should emit blocks of a specified size to reduce fragmentation. This means the goal is not always lowest latency but little overhead and then lowest latency
// syntax inspired by https://yew.rs/docs/concepts/basic-web-technologies/html

type TemplatePart = ();

// maybe this just also becomes a macro?
// maybe we can create a basic macro_rules macro that works but is not efficient?
fn main(title: TemplatePart, inner: TemplatePart) {
    html! {
        <html>
            <head>
                <title>{title}</title>
            </head>
            <body>
                partial!(inner)
            </body>
        </html>
    }
}

html! {
    template!(main(html! { {dynamically_calculate_title()} },
        html! {
            <div class=["hi "{ test }]>
                {
                    let test = get_version();
                }
                "hi what :-)"
                {
                    let result = fetch_database_row().await;
                }
                // maybe accept normal syntax but just in a really specific form
                foreach! (result, |row| html! {
                    <li>
                        { row.name }
                    </li>
                })
                if! (condition, html! {
                    "true"
                }, html! {
                    "false"
                })
            </div>
        }
    ))
}

/*
// https://docs.rs/http-body/latest/http_body/trait.Body.html
pub fn output(
) -> impl for<'a> Coroutine<&'a mut std::task::Context<'a>, Yield = Frame<impl Buf>, Return = ()> {
    async || {
        yield Frame::data(&b"test"[..]);
    }
}
*/
pub struct TemplateHttpBody<S: Stream<Item = Bytes>> {
    stream: S,
    chunk_size: usize,
}

impl<S: Stream<Item = Bytes>> Body for TemplateHttpBody<S> {
    type Data = &'static [u8];

    type Error = Infallible;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        todo!()
    }
}
