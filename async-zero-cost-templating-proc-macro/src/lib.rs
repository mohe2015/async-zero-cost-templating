use async_zero_cost_templating_proc_macro2::{
    codegen::{codegen, top_level},
    intermediate::{simplify, Intermediate},
    parse::{top_level_parse, HtmlChildren},
};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn html_proc_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (html_children, diagnostics) = top_level_parse(input.into());
    let intermediate = html_children
        .into_iter()
        .flat_map(Vec::<Intermediate>::from)
        .collect();
    let intermediate = simplify(intermediate);

    let output = top_level(intermediate);
    quote! {
        #diagnostics
        #output
    }
    .into()
}
