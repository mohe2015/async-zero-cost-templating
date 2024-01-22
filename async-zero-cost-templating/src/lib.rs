mod future_to_stream;

pub use async_zero_cost_templating_proc_macro::html_proc_macro;
pub use future_to_stream::FutureToStream;

use std::convert::Infallible;

use bytes::Bytes;
use futures_core::Stream;

use http_body::{Body, Frame};

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

macro_rules! html {
    ($($tt: tt)*) => {};
}

// it should emit blocks of a specified size to reduce fragmentation. This means the goal is not always lowest latency but little overhead and then lowest latency
// syntax inspired by https://yew.rs/docs/concepts/basic-web-technologies/html

pub type TemplatePart = ();

// maybe this just also becomes a macro?
// maybe we can create a basic macro_rules macro that works but is not efficient?
pub fn main(_title: TemplatePart, _inner: TemplatePart) {
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
            <div class=["hi "{ test }] id="test">
                {
                    let test = get_version();
                }
                "hi what :-)"
                {
                    let result = fetch_database_row().await;
                }
                // maybe accept normal syntax but just in a really specific form
                for row in result {
                    <li>
                        { row.name }
                    </li>
                }
                if condition {
                    "true"
                    { test }
                } else {
                    "false"
                }
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

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use futures_util::StreamExt as _;

    use crate::{future_to_stream::TheStream, stream_example};

    #[tokio::test]
    pub async fn test1() {
        let stream = TheStream::new(stream_example);
        assert_eq!(core::mem::size_of_val(&stream), 1);

        let mut stream = pin!(stream);
        while let Some(value) = stream.next().await {
            eprintln!("got {}", value)
        }
        eprintln!("done")
    }
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/compile_fail/*.rs");
    t.pass("tests/ui/pass/*.rs");
}
