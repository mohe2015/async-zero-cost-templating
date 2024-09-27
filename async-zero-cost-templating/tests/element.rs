extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::TemplateToStream;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let value = alloc::borrow::Cow::Borrowed("hi");
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let future = async move {
        html! {
            <a href=["test" (value)]>"Link"</a>
        }
    };
    let stream = pin!(TemplateToStream::new(future, rx));
    let result: String = stream.collect().await;
    assert_eq!(result, r#"<a href="testhi">Link</a>"#)
}
