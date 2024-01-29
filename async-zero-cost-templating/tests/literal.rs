extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::TheStream;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let stream = html! {
        "hello world"
    };
    let mut stream = pin!(TheStream::new(stream));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
