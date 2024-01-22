use crate::{
    intermediate::Intermediate,
    parse::{HtmlForLoop, HtmlIf},
};
use proc_macro2::Literal;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub fn codegen(input: Vec<Intermediate>) -> proc_macro2::TokenStream {
    let inner = input.into_iter().map(codegen_intermediate);
    quote! {
        let _ = |stream: ::async_zero_cost_templating::FutureToStream| async move {
            #(#inner)*
        };
    }
}
pub fn codegen_intermediate(input: Intermediate) -> proc_macro2::TokenStream {
    match input {
        Intermediate::Literal(lit, span) => {
            let byte_string = Literal::byte_string(lit.as_bytes());
            quote_spanned! {span=>
                stream._yield(::bytes::Bytes::from_static(#byte_string)).await;
            }
        }
        Intermediate::Computed(computed) => {
            let span = computed.span();
            quote_spanned! {span=>
                stream._yield(#computed).await;
            }
        }
        Intermediate::If(HtmlIf {
            if_token,
            cond,
            then_branch,
            else_branch,
        }) => {
            let else_ = else_branch.map(|(else_, brace, inner)| {
                let inner = codegen(inner);
                quote! {
                    #else_ {
                        #inner
                    }
                }
            });
            let inner = codegen(then_branch.1);
            quote! {
                #if_token #cond {
                    #inner
                } #else_
            }
        }
        Intermediate::For(HtmlForLoop {
            for_token,
            pat,
            in_token,
            expr,
            body,
        }) => {
            let inner = codegen(body.1);
            quote! {
                #for_token #pat #in_token #expr {
                    #inner
                }
            }
        }
    }
}
