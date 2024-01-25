use async_zero_cost_templating_proc_macro::html_proc_macro;

pub fn main() {
    let _ = html_proc_macro! {
        <!doctype html>
    };
}
