use async_zero_cost_templating::html_proc_macro;
use async_zero_cost_templating::TheStream;
use bytes::Bytes;
use core::pin::pin;
use futures_util::stream::StreamExt;
use std::io::Write;

#[tokio::test]
async fn test() {
    let stream = html_proc_macro! {
        "hello world"
    };
    let mut stream = pin!(TheStream::new(stream));
    let mut stdout = std::io::stdout().lock();
    while let Some(element) = stream.next().await {
        stdout.write_all(&element).unwrap();
    }
    stdout.write_all(b"\n").unwrap();
}
