extern crate alloc;

use async_zero_cost_templating::{html, TemplateToStream};
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let mut result = futures_util::stream::iter([
        alloc::borrow::Cow::Borrowed("abc"),
        alloc::borrow::Cow::Borrowed("def"),
        alloc::borrow::Cow::Borrowed("ghi"),
    ]);
    let stream = html! {
        while let Some(row) = result.next().await {
            "true"
            ( row )
        }
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"trueabctruedeftrueghi"#)
}
