use proc_macro2::Span;
use syn::{token::Brace, Block, Expr, Pat, Token};

use crate::parse::{HtmlChildren, HtmlElement, HtmlForLoop, HtmlIf};

pub enum Intermediate {
    Literal(String, Span),
    Computed(Block),
    If(HtmlIf<Vec<Intermediate>>),
    For(HtmlForLoop<Vec<Intermediate>>),
}

pub fn to_intermediate(input: HtmlChildren) -> Vec<Intermediate> {
    input
        .children
        .into_iter()
        .flat_map(|child| match child {
            crate::parse::Html::Literal(literal) => {
                Vec::from([Intermediate::Literal(literal.value(), literal.span())])
            }
            crate::parse::Html::Computed(computed) => Vec::from([Intermediate::Computed(computed)]),
            crate::parse::Html::If(HtmlIf {
                if_token,
                cond,
                then_branch,
                else_branch,
            }) => Vec::from([Intermediate::If(HtmlIf {
                if_token,
                cond,
                then_branch: (then_branch.0, to_intermediate(then_branch.1)),
                else_branch: else_branch.map(|else_branch| {
                    (else_branch.0, else_branch.1, to_intermediate(else_branch.2))
                }),
            })]),
            crate::parse::Html::For(HtmlForLoop {
                for_token,
                pat,
                in_token,
                expr,
                body,
            }) => Vec::from([Intermediate::For(HtmlForLoop {
                for_token,
                pat,
                in_token,
                expr,
                body: (body.0, to_intermediate(body.1)),
            })]),
            crate::parse::Html::Element(HtmlElement {
                open_start,
                open_tag_name,
                attributes,
                open_end,
                children,
                close,
            }) => Vec::from([
                Intermediate::Literal("<".to_owned(), open_start.span),
                Intermediate::Literal(open_tag_name.to_string(), open_tag_name.span()),
            ]),
        })
        .collect()
}
