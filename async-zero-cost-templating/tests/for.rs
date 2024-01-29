extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let mut result = futures_util::stream::iter([
        alloc::borrow::Cow::Borrowed("abc"),
        alloc::borrow::Cow::Borrowed("def"),
        alloc::borrow::Cow::Borrowed("ghi"),
    ]);
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let future = async move {
        html! {
            while let Some(row) = result.next().await {
                "true"
                ( row )
            }
        }
    };
    let mut stream = pin!(TemplateToStream::new(future, rx));
    while let Some(value) = stream.next().await {
        print!("{}", value)
    }
    println!();
}
