extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::TheStream;
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
        for row in &mut result {
            "true"
            ( row )
        }
    };
    let mut stream = pin!(TheStream::new(stream));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
