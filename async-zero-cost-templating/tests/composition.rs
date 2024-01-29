extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::FutureToStream;
use async_zero_cost_templating::TheStream;
use futures_core::Future;
use core::pin::pin;
use std::borrow::Cow;
use std::cell::Cell;
use futures_util::stream::StreamExt;

pub fn composition<'b: 'a, 'a>(future_to_stream: &'b FutureToStream<Cow<'a, str>>, value: &'a str) -> impl Future<Output = ()> + 'a {
    html! {
        <a href=["test" (Cow::Borrowed(value))]>"Link"</a>
    }
}

#[tokio::test]
async fn test() {
    let value = alloc::borrow::Cow::Borrowed("hi");
    let future_to_stream = FutureToStream(Cell::new(None));
    let future_to_stream = &future_to_stream;
    let future = html! {
        <a href=["test" (value)]>"Link"</a>
        {
            composition(future_to_stream, "hi").await
        }
    };
    let mut stream = pin!(TheStream::new(future_to_stream, future));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
