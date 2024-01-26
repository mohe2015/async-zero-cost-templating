use async_zero_cost_templating::html_proc_macro;
use async_zero_cost_templating::TheStream;
use bytes::Bytes;
use core::pin::pin;
use std::io::Write;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let value = Bytes::from_static(b"hi");
    let stream = html_proc_macro! {
        <a href=["test" {value}]>"Link"</a>
    };
    let mut stream = pin!(TheStream::new(stream));
    let mut stdout = std::io::stdout().lock();
    while let Some(element) = stream.next().await {
        stdout.write_all(&element).unwrap();
    }
    stdout.write_all(b"\n").unwrap();
}
