use async_zero_cost_templating::html_proc_macro;
use async_zero_cost_templating::TheStream;
use bytes::Bytes;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let variable = Bytes::from_static(b"hi");
    let mut test = pin!(TheStream::new(html_proc_macro! {
        { variable }
    }));
    while let Some(element) = test.next().await {
        println!("{:?}", element);
    }
}
