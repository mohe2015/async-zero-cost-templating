extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::TemplateToStream;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let value = alloc::borrow::Cow::Borrowed("hi");
    let stream = html! {
        <a href=["test" (value)]>"Link"</a>
    };
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<a href="testhi">Link</a>"#)
}
