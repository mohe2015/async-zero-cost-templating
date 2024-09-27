extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let condition = false;
    let stream = html! {
        <input if condition {
            value="/test"
        } else {
            class="error"
        }>
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<input class="error">"#)
}
