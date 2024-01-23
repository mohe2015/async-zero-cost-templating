use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{spanned::Spanned, token::Brace, Block};

use crate::parse::{
    Html, HtmlAttribute, HtmlAttributeValue, HtmlChildren, HtmlElement, HtmlForLoop, HtmlIf,
};

pub enum Intermediate {
    Literal(String, Span),
    Computed((Brace, TokenStream)),
    If(HtmlIf<Vec<Intermediate>>),
    For(HtmlForLoop<Vec<Intermediate>>),
}

impl From<HtmlAttributeValue> for Vec<Intermediate> {
    fn from(value: HtmlAttributeValue) -> Self {
        value
            .children
            .into_iter()
            .flat_map(Vec::<Intermediate>::from)
            .collect()
    }
}

impl From<HtmlAttribute> for Vec<Intermediate> {
    fn from(value: HtmlAttribute) -> Self {
        Vec::from_iter(
            [Intermediate::Literal(
                value.key.to_string(),
                value.key.span(),
            )]
            .into_iter()
            .chain(
                value
                    .value
                    .map(|value| {
                        [Intermediate::Literal("=".to_owned(), value.0.span())]
                            .into_iter()
                            .chain(Vec::<Intermediate>::from(value.1))
                    })
                    .into_iter()
                    .flatten(),
            ),
        )
    }
}

impl<T: Into<Vec<Intermediate>>> From<Html<T>> for Vec<Intermediate> {
    fn from(value: Html<T>) -> Self {
        match value {
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
                then_branch: (then_branch.0, then_branch.1.into()),
                else_branch: else_branch
                    .map(|else_branch| (else_branch.0, else_branch.1, else_branch.2.into())),
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
                body: (body.0, body.1.into()),
            })]),
            crate::parse::Html::Element(HtmlElement {
                open_start,
                open_tag_name,
                attributes,
                open_end,
                children,
            }) => Vec::from_iter(
                [
                    Intermediate::Literal("<".to_owned(), open_start.span),
                    Intermediate::Literal(open_tag_name.to_string(), open_tag_name.span()),
                ]
                .into_iter()
                .chain(attributes.into_iter().flat_map(Vec::<Intermediate>::from))
                .chain([Intermediate::Literal("<".to_owned(), open_end.span)])
                .chain(
                    children
                        .map(|children| {
                            Vec::<Intermediate>::from(children.0).into_iter().chain([
                                Intermediate::Literal("<".to_owned(), children.1.span()),
                                Intermediate::Literal("/".to_owned(), children.2.span()),
                                Intermediate::Literal(children.3.to_string(), children.3.span()),
                                Intermediate::Literal(">".to_owned(), children.4.span()),
                            ])
                        })
                        .into_iter()
                        .flatten(),
                ),
            ),
        }
    }
}

impl From<HtmlChildren> for Vec<Intermediate> {
    fn from(value: HtmlChildren) -> Self {
        value
            .children
            .into_iter()
            .flat_map(Vec::<Intermediate>::from)
            .collect()
    }
}

pub fn simplify(input: Vec<Intermediate>) -> Vec<Intermediate> {
    let (mut acc, current) =
        input
            .into_iter()
            .fold((Vec::new(), None), |(mut acc, current), next| {
                match (current, next) {
                    (None, Intermediate::Literal(lit, span)) => (acc, Some((lit, span))),
                    (Some((lit1, span1)), Intermediate::Literal(lit2, span2)) => {
                        (acc, Some((lit1 + &lit2, span1.join(span2).unwrap())))
                    }
                    (Some((lit, span)), next) => (
                        {
                            acc.push(Intermediate::Literal(lit, span));
                            acc.push(next);
                            acc
                        },
                        None,
                    ),
                    (None, next) => (
                        {
                            acc.push(next);
                            acc
                        },
                        None,
                    ),
                }
            });
    if let Some((lit, span)) = current {
        acc.push(Intermediate::Literal(lit, span));
    }
    acc
}
