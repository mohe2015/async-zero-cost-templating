extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::FutureToStream;
use async_zero_cost_templating::TheStream;
use core::pin::pin;
use futures_util::stream::StreamExt;
use std::cell::Cell;

#[tokio::test]
async fn test() {
    let variable = alloc::borrow::Cow::Borrowed("hi");
    let future_to_stream = FutureToStream(Cell::new(None));
    let future_to_stream = &future_to_stream;
    let future = html! {
        "hello world"
        ( variable )
    };
    let mut stream = pin!(TheStream::new(future_to_stream, future));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
