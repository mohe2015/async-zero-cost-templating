use async_zero_cost_templating_proc_macro::html_proc_macro;
use bytes::Bytes;

pub fn main() {
    let variable = Bytes::from_static(b"hi");
    html_proc_macro! {
        "hello world"
        { variable }
    }
}
