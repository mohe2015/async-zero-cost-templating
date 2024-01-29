extern crate alloc;

use async_zero_cost_templating::html;
use core::pin::pin;
use futures_core::Future;
use std::borrow::Cow;
use tokio::select;

pub fn composition<'a, 'b, 'c: 'a>(
    tx: tokio::sync::mpsc::Sender<Cow<'a, str>>,
    value: &'c str,
) -> impl Future<Output = ()> + 'a {
    html! {
        <a href=["test" (Cow::Borrowed(value))]>"Link"</a>
    }
}

#[tokio::test]
async fn test() {
    let value = String::from("hello world");
    let value = &value;
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut future = pin!(html! {
        <h1>"Test"</h1>
        {
            composition(tx, value).await
        }
    });

    println!();
}
