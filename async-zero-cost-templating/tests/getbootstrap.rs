extern crate alloc;

use async_zero_cost_templating::html;
use async_zero_cost_templating::FutureToStream;
use async_zero_cost_templating::TheStream;
use core::pin::pin;
use futures_util::stream::StreamExt;
use std::cell::Cell;

#[tokio::test]
async fn test() {
    let title = async { alloc::borrow::Cow::Borrowed("Bootstrap demo") };
    let future_to_stream = FutureToStream(Cell::new(None));
    let future_to_stream = &future_to_stream;
    let mut result = futures_util::stream::iter([
        alloc::borrow::Cow::Borrowed("abc"),
        alloc::borrow::Cow::Borrowed("def"),
        alloc::borrow::Cow::Borrowed("ghi"),
    ]);
    let morning = false;
    let future = html! {
        <!doctype html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title>( title.await )</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-T3c6CoIi6uLrA9TneNEoa7RxnatzjcDSCmG1MXxSR1GAsXEV/Dwwykc2MPK8M2HN" crossorigin="anonymous">
        </head>
        <body>
            <h1>
                if morning {
                    "Good morning!"
                } else {
                    "Good night!"
                }
            </h1>
            <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js" integrity="sha384-C6RzsynM9kWDrMNeT87bh95OGNyZPhcTNXj1NW7RuBCsyN/o0jlpcV8Qyq46cDfL" crossorigin="anonymous"></script>
            <ul>
            for row in &mut result {
                <li>
                    ( row )
                </li>
            }
            </ul>
        </body>
        </html>
    };
    let mut stream = pin!(TheStream::new(future_to_stream, future));
    while let Some(element) = stream.next().await {
        print!("{}", element);
    }
    println!();
}
