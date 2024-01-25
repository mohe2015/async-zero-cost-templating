use async_zero_cost_templating_proc_macro::html_proc_macro;
use bytes::Bytes;

pub fn main() {
    let mut result = futures_util::stream::iter([Bytes::from_static(b"hi"), Bytes::from_static(b"hi"), Bytes::from_static(b"hi")]);
    html_proc_macro! {
        for row in &mut result {
            "true"
            { row }
        }
    }
}
