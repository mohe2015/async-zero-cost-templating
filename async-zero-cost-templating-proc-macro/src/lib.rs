use async_zero_cost_templating_proc_macro2::parse::HtmlChildren;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn html_proc_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _html_children = parse_macro_input!(input as HtmlChildren);

    quote! {}.into()
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/compile_fail/*.rs");
    t.pass("tests/ui/pass/*.rs");
}
