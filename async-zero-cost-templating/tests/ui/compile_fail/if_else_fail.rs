use async_zero_cost_templating::html;

pub fn main() {
    html! {
        if condition {
            <button type="submit"></button>
        }
    }
}
