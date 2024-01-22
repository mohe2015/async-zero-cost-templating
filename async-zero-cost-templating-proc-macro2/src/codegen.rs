use crate::intermediate::Intermediate;
use quote::quote;

pub fn codegen(_input: Vec<Intermediate>) -> proc_macro2::TokenStream {
    quote! {
        let _ = |stream: ::async_zero_cost_templating::FutureToStream| async move {
            stream._yield(1).await;
            stream._yield(2).await;
            stream._yield(3).await;
        };
    }
}
