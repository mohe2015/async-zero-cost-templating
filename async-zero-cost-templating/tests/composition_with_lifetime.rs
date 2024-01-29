extern crate alloc;

use async_zero_cost_templating::html;
use futures_core::Future;
use tokio::select;
use core::pin::pin;
use std::borrow::Cow;
use std::cell::Cell;
use futures_util::stream::StreamExt;

// c must live longer than a
// b must live longer than a?
pub fn composition<'a, 'b, 'c: 'a>(tx: tokio::sync::mpsc::Sender<Cow<'a, str>>, value: &'c str) -> impl Future<Output = ()> + 'a {
    html! {
        <a href=["test" (Cow::Borrowed(value))]>"Link"</a>
    }
}

#[tokio::test]
async fn test() {
    let value = String::from("hello world");
    let value = &value;
    let value = "static";
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut future = pin!(html! {
        <h1>"Test"</h1>
        {
            composition(tx, value).await
        }
    });
    loop {
        select! {
            _ = &mut future => {
                // never resume a completed future
                break;
            },
            Some(value) = rx.recv() => {
                print!("{}", value);
            }
            else => break
        }
    }
    while let Some(value) = rx.recv().await {
        print!("{}", value);
    }
    println!();
}
