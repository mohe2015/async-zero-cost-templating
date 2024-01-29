extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::TemplateToStream;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let value = alloc::borrow::Cow::Borrowed("hi");
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let future = html! {
        <a href=["test" (value)]>"Link"</a>
    };
    let mut stream = pin!(TemplateToStream::new(future, rx));
    while let Some(value) = stream.next().await {
        print!("{}", value)
    }
    println!();
}
