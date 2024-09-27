extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let condition = false;
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let future = async move {
        html! {
            <input if condition {
                value="/test"
            } else {
                class="error"
            }>
        }
    };
    let stream = pin!(TemplateToStream::new(future, rx));
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<input class="error">"#)
}
