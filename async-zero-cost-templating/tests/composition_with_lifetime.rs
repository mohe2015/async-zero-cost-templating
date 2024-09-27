extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_core::Future;
use futures_util::StreamExt as _;
use std::borrow::Cow;

pub fn composition<'a>(
    tx: tokio::sync::mpsc::Sender<Cow<'a, str>>,
    value: &'a str,
) -> impl Future<Output = ()> + 'a {
    async move {
        html! {
            <a href=["test" (Cow::Borrowed(value))]>"Link"</a>
        }
    }
}

#[tokio::test]
async fn test() {
    let value = String::from("hello world");
    let value = &value;
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let future = async move {
        html! {
            <h1>"Test"</h1>
            {
                composition(tx, value).await
            }
        }
    };
    let stream = pin!(TemplateToStream::new(future, rx));
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<h1>Test</h1><a href="testhello world">Link</a>"#)
}
