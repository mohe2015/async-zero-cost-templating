use async_zero_cost_templating_proc_macro::html_proc_macro;

pub fn main() {
    html_proc_macro! {
        for row in result {
            "true"
            { variable }
        }
    }
}
