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
    let value = String::from("hello world");
    inner(&value).await;
}

async fn inner<'a>(value: &'a str) {
    let future_to_stream = FutureToStream(Cell::new(None));
    let future_to_stream = &future_to_stream;
    let future = html! {
        <h1>"Test"</h1>
        {
            composition(future_to_stream, value).await
        }
    };
    let mut stream = pin!(TheStream::new(future_to_stream, future));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
