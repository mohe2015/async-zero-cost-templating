extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let stream = html! {
        "hello world"
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"hello world"#)
}
