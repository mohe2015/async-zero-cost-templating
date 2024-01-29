extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::FutureToStream;
use async_zero_cost_templating::TheStream;
use core::pin::pin;
use std::cell::Cell;
use futures_util::stream::StreamExt;

// should the future be sync and send?
#[tokio::test]
async fn test() {
    let future_to_stream = FutureToStream(Cell::new(None));
    let stream = html! {
        <!doctype html>
    };
    let future = stream(&future_to_stream);
    let mut stream = pin!(TheStream::new(&future_to_stream, future));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
