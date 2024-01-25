use async_zero_cost_templating::html_proc_macro;
use async_zero_cost_templating::TheStream;
use bytes::Bytes;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let condition = true;
    let variable = Bytes::from_static(b"hi");
    let stream = html_proc_macro! {
        if condition {
            "true"
            { variable }
        } else {
            "false"
            { variable }
        }
    };
    let mut stream = pin!(TheStream::new(stream));
    while let Some(element) = stream.next().await {
        println!("{:?}", element);
    }
}
