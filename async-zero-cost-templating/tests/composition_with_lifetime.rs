extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_core::{Future, Stream};
use futures_util::StreamExt as _;
use std::borrow::Cow;

pub fn composition<'a>(
    value: &'a str,
) -> TemplateToStream<Cow<'a, str>, impl Future<Output = ()> + 'a> {
    html! {
        <a href=["test" (Cow::Borrowed(value))]>"Link"</a>
    }
}

#[tokio::test]
async fn test() {
    let value = String::from("hello world");
    let value = &value;
    let stream = html! {
        <h1>"Test"</h1>
        {
            composition(value)
        }
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<h1>Test</h1><a href="testhello world">Link</a>"#)
}
