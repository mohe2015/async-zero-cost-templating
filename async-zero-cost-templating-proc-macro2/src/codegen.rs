use crate::{
    intermediate::Intermediate,
    parse::{HtmlForLoop, HtmlIf, HtmlWhile},
};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub fn top_level(input: Vec<Intermediate>) -> proc_macro2::TokenStream {
    let inner = codegen(input);
    quote! {
        {
            let (tx, rx) = ::tokio::sync::mpsc::channel(1);
            let future = async move {
                #inner
            };
            TemplateToStream::new(future, rx)
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
                tx.send(::alloc::borrow::Cow::Borrowed(#lit)).await.unwrap();
            }
        }
        Intermediate::ComputedValue((_brace, computed_value)) => {
            let span = computed_value.span();
            quote_spanned! {span=>
                tx.send(#computed_value).await.unwrap();
            }
        }
        Intermediate::Computation((_brace, computation)) => {
            let span = computation.span();
            quote_spanned! {span=>
                #computation
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
        Intermediate::While(HtmlWhile {
            while_token,
            cond,
            body,
        }) => {
            let inner = codegen(body.1);
            quote! {
                #while_token #cond {
                    #inner
                }
            }
        }
    }
}
