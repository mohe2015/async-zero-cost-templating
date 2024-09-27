extern crate alloc;

use async_zero_cost_templating::html;

pub fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let _ = async move {
        html! {
            <!doctype html>
            <html
        }
    };
}
