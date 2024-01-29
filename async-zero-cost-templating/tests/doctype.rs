extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::FutureToStream;
use async_zero_cost_templating::TheStream;
use core::pin::pin;
use std::cell::Cell;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let stream = html! {
        <!doctype html>
    };
    let future_to_stream = FutureToStream(Cell::new(None));
    let mut stream = pin!(TheStream::new(&future_to_stream, stream(&future_to_stream)));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
