#![feature(coroutines)]
#![feature(coroutine_trait)]

use std::{convert::Infallible, ops::Coroutine};

use bytes::{Buf, Bytes};
use http_body::{Body, Frame};

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

pub fn generated_code() -> impl Coroutine<(), Yield = Bytes, Return = ()> {
    || {
        yield Bytes::from_static(br#"<div class="#);
    }
}

// https://docs.rs/http-body/latest/http_body/trait.Body.html
pub fn output(
) -> impl for<'a> Coroutine<&'a mut std::task::Context<'a>, Yield = Frame<impl Buf>, Return = ()> {
    |cx: &'a mut std::task::Context<'a>| {
        yield Frame::data(&b"test"[..]);
    }
}

pub struct TemplateHttpBody<C: Coroutine<(), Yield = Bytes, Return = ()>> {
    coroutine: C,
    chunk_size: usize,
}

impl<C: Coroutine<(), Yield = Bytes, Return = ()>> Body for TemplateHttpBody<C> {
    type Data = &'static [u8];

    type Error = Infallible;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        todo!()
    }
}
