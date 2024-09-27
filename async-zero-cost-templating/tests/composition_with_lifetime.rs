extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_core::{Future, Stream};
use futures_util::StreamExt as _;
use std::borrow::Cow;

pub fn composition<'a>(
    value: &'a str,
) -> impl Stream<Item = Cow<'a, str>> + 'a {
    html! {
        <a href=["test" (Cow::Borrowed(value))]>"Link"</a>
    }
}

#[tokio::test]
async fn test() {
    let value = String::from("hello world");
    let value = &value;
    let future = html! {
        <h1>"Test"</h1>
        {
            composition(tx, value)
        }
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<h1>Test</h1><a href="testhello world">Link</a>"#)
}
