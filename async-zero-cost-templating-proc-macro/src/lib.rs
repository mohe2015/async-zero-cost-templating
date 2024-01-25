use async_zero_cost_templating_proc_macro2::{
    parse::{top_level_parse},
};

#[proc_macro]
pub fn html_proc_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    top_level_parse(input.into()).into()
}
