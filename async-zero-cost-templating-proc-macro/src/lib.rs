use async_zero_cost_templating_proc_macro2::{
    codegen::codegen,
    intermediate::{simplify, Intermediate},
    parse::HtmlChildren,
};
use syn::parse_macro_input;

#[proc_macro]
pub fn html_proc_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let html_children = parse_macro_input!(input as HtmlChildren);
    let intermediate = Vec::<Intermediate>::from(html_children);
    let intermediate = simplify(intermediate);

    codegen(intermediate).into()
}
