use crate::{
    intermediate::Intermediate,
    parse::{HtmlForLoop, HtmlIf},
};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub fn top_level(input: Vec<Intermediate>) -> proc_macro2::TokenStream {
    let inner = codegen(input);
    quote! {
        |stream: &::async_zero_cost_templating::FutureToStream<alloc::borrow::Cow<'a, str>>| async move {
            #inner
        }
    }
}

pub fn codegen(input: Vec<Intermediate>) -> proc_macro2::TokenStream {
    let inner = input.into_iter().map(codegen_intermediate);
    quote! {
        #(#inner)*
    }
}
pub fn codegen_intermediate(input: Intermediate) -> proc_macro2::TokenStream {
    match input {
        Intermediate::Literal(lit, span) => {
            quote_spanned! {span=>
                stream._yield(::alloc::borrow::Cow::Borrowed(#lit)).await;
            }
        }
        Intermediate::ComputedValue((_brace, computed_value)) => {
            let span = computed_value.span();
            quote_spanned! {span=>
                stream._yield(#computed_value).await;
            }
        }
        Intermediate::Computation((_brace, computed)) => {
            let span = computed.span();
            quote_spanned! {span=>
                let () = { #computed };
            }
        }
        Intermediate::If(HtmlIf {
            if_token,
            cond,
            then_branch,
            else_branch,
        }) => {
            let else_ = else_branch.map(|(else_, _brace, inner)| {
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
            for_token: _,
            pat,
            in_token: _,
            expr,
            body,
        }) => {
            let inner = codegen(body.1);
            quote! {
                let __stream = #expr;
                // TODO FIXME import from our crate to ensure it exists, maybe also just replace our for with the while let
                while let Some(#pat) = ::futures_util::StreamExt::next(__stream).await {
                    #inner
                }
            }
        }
    }
}
