extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_util::stream::StreamExt;

// should the future be sync and send?
#[tokio::test]
async fn test() {
    let stream = html! {
        <!doctype html>
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<!doctype html>"#)
}
