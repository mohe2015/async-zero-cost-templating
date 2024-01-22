use async_zero_cost_templating_proc_macro::html_proc_macro;

pub fn main() {
    let result = futures_util::stream::iter([1, 2, 3]);
    html_proc_macro! {
        for row in &mut result {
            "true"
            { row }
        }
    }
}
