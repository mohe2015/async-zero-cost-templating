use async_zero_cost_templating_proc_macro::html_proc_macro;

html_proc_macro! {
    if test {
        "true"
        { variable }
    }
}

pub fn main() {}
