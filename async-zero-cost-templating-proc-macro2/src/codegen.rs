use crate::parse::HtmlChildren;
use quote::quote;

pub fn codegen(input: HtmlChildren) -> proc_macro2::TokenStream {
    quote! {
        |stream: ::async_zero_cost_templating::FutureToStream| async move {
            stream._yield(1).await;
            stream._yield(2).await;
            stream._yield(3).await;
        };
    }
}
