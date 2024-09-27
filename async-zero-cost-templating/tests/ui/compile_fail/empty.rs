extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::TemplateToStream;
use core::pin::pin;
use futures_util::stream::StreamExt;
use std::borrow::Cow;

#[tokio::test]
async fn test() {
    let stream = html! {};
    let result: String = stream.collect().await;
    assert_eq!(result, r#""#)
}
