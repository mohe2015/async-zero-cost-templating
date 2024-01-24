use async_zero_cost_templating_proc_macro2::{
    codegen::{codegen, top_level},
    intermediate::{simplify, Intermediate},
    parse::{top_level_parse, HtmlChildren},
};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn html_proc_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let (html_children, diagnostics) = top_level_parse(input.into());
    let intermediate = Vec::<Intermediate>::from(html_children);
    let intermediate = simplify(intermediate);

    let output = top_level(intermediate);
    quote! {
        #diagnostics
        #output
    }
    .into()
}
