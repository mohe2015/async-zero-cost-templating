use async_zero_cost_templating::html_proc_macro;
use async_zero_cost_templating::TheStream;
use bytes::Bytes;
use core::pin::pin;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let mut result = futures_util::stream::iter([
        Bytes::from_static(b"hi"),
        Bytes::from_static(b"hi"),
        Bytes::from_static(b"hi"),
    ]);
    let stream = html_proc_macro! {
        for row in &mut result {
            "true"
            { row }
        }
    };
    let mut stream = pin!(TheStream::new(stream));
    while let Some(element) = stream.next().await {
        println!("{:?}", element);
    }
}
