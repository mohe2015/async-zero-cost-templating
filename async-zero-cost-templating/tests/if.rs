extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::TemplateToStream;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let condition = true;
    let variable = alloc::borrow::Cow::Borrowed("hi");
    let stream = html! {
        if condition {
            "true"
            ( variable )
        }
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"truehi"#)
}
