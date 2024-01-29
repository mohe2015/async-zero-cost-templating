extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let condition = false;
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let future = html! {
        <input if condition {
            value="/test"
        } else {
            class="error"
        }>
    };
    let mut stream = pin!(TemplateToStream::new(future, rx));
    while let Some(value) = stream.next().await {
        print!("{}", value)
    }
    println!();
}
