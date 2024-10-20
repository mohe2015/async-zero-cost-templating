use itertools::Itertools as _;
use proc_macro2::{Span, TokenStream};
use syn::{
    spanned::Spanned,
    token::{Brace, Paren},
};

use crate::parse::{
    DashOrColon, HtmlElement, HtmlForLoop, HtmlIf, HtmlInAttributeContext, HtmlInAttributeValueContext, HtmlInElementContext, HtmlWhile
};

pub enum Intermediate {
    Literal(String, Span),
    Computation((Brace, TokenStream)),
    ComputedValue((Paren, TokenStream)),
    If(HtmlIf<Vec<Intermediate>>),
    While(HtmlWhile<Vec<Intermediate>>),
    For(HtmlForLoop<Vec<Intermediate>>),
}

impl From<HtmlInAttributeContext> for Vec<Intermediate> {
    fn from(value: HtmlInAttributeContext) -> Self {
        match value {
            HtmlInAttributeContext::Literal(key, value) => Vec::from_iter(
                [Intermediate::Literal(
                    " ".to_owned() + &key.pairs()
                    .map(|p| {
                        p.value().to_string()
                            + match p.punct() {
                                Some(DashOrColon::Colon(_)) => ":",
                                Some(DashOrColon::Dash(_)) => "-",
                                None => "",
                            }
                    })
                    .join(""),
                    key.first().unwrap().span(),
                )]
                .into_iter()
                .chain(
                    value
                        .map(|value| {
                            [Intermediate::Literal(r#"=""#.to_owned(), value.0.span())]
                                .into_iter()
                                .chain(value.1.into_iter().flat_map(Vec::<Intermediate>::from))
                                .chain([Intermediate::Literal(r#"""#.to_owned(), value.0.span())])
                        })
                        .into_iter()
                        .flatten(),
                ),
            ),
            HtmlInAttributeContext::Computation(computation) => {
                Vec::from([Intermediate::Computation(computation)])
            }
            HtmlInAttributeContext::If(HtmlIf {
                if_token,
                cond,
                then_branch,
                else_branch,
            }) => Vec::from([Intermediate::If(HtmlIf {
                if_token,
                cond,
                then_branch: (
                    then_branch.0,
                    then_branch
                        .1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
                else_branch: else_branch.map(|else_branch| {
                    (
                        else_branch.0,
                        else_branch.1,
                        else_branch
                            .2
                            .into_iter()
                            .flat_map(Vec::<Intermediate>::from)
                            .collect(),
                    )
                }),
            })]),
            HtmlInAttributeContext::For(HtmlForLoop {
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
                body: (
                    body.0,
                    body.1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
            })]),
            crate::parse::HtmlInAttributeContext::While(HtmlWhile {
                while_token,
                cond,
                body,
            }) => Vec::from([Intermediate::While(HtmlWhile {
                while_token,
                cond,
                body: (
                    body.0,
                    body.1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
            })]),
        }
    }
}

impl From<HtmlInAttributeValueContext> for Vec<Intermediate> {
    fn from(value: HtmlInAttributeValueContext) -> Self {
        match value {
            crate::parse::HtmlInAttributeValueContext::Literal(literal) => {
                Vec::from([Intermediate::Literal(literal.value(), literal.span())])
            }
            crate::parse::HtmlInAttributeValueContext::ComputedValue(computed_value) => {
                Vec::from([Intermediate::ComputedValue(computed_value)])
            }
            crate::parse::HtmlInAttributeValueContext::Computation(computation) => {
                Vec::from([Intermediate::Computation(computation)])
            }
            crate::parse::HtmlInAttributeValueContext::If(HtmlIf {
                if_token,
                cond,
                then_branch,
                else_branch,
            }) => Vec::from([Intermediate::If(HtmlIf {
                if_token,
                cond,
                then_branch: (
                    then_branch.0,
                    then_branch
                        .1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
                else_branch: else_branch.map(|else_branch| {
                    (
                        else_branch.0,
                        else_branch.1,
                        else_branch
                            .2
                            .into_iter()
                            .flat_map(Vec::<Intermediate>::from)
                            .collect(),
                    )
                }),
            })]),
            crate::parse::HtmlInAttributeValueContext::For(HtmlForLoop {
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
                body: (
                    body.0,
                    body.1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
            })]),
            crate::parse::HtmlInAttributeValueContext::While(HtmlWhile {
                while_token,
                cond,
                body,
            }) => Vec::from([Intermediate::While(HtmlWhile {
                while_token,
                cond,
                body: (
                    body.0,
                    body.1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
            })]),
        }
    }
}

impl From<HtmlInElementContext> for Vec<Intermediate> {
    fn from(value: HtmlInElementContext) -> Self {
        match value {
            crate::parse::HtmlInElementContext::Literal(literal) => {
                Vec::from([Intermediate::Literal(literal.value(), literal.span())])
            }
            crate::parse::HtmlInElementContext::ComputedValue(computed_value) => {
                Vec::from([Intermediate::ComputedValue(computed_value)])
            }
            crate::parse::HtmlInElementContext::Computation(computation) => {
                Vec::from([Intermediate::Computation(computation)])
            }
            crate::parse::HtmlInElementContext::If(HtmlIf {
                if_token,
                cond,
                then_branch,
                else_branch,
            }) => Vec::from([Intermediate::If(HtmlIf {
                if_token,
                cond,
                then_branch: (
                    then_branch.0,
                    then_branch
                        .1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
                else_branch: else_branch.map(|else_branch| {
                    (
                        else_branch.0,
                        else_branch.1,
                        else_branch
                            .2
                            .into_iter()
                            .flat_map(Vec::<Intermediate>::from)
                            .collect(),
                    )
                }),
            })]),
            crate::parse::HtmlInElementContext::For(HtmlForLoop {
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
                body: (
                    body.0,
                    body.1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
            })]),
            crate::parse::HtmlInElementContext::While(HtmlWhile {
                while_token,
                cond,
                body,
            }) => Vec::from([Intermediate::While(HtmlWhile {
                while_token,
                cond,
                body: (
                    body.0,
                    body.1
                        .into_iter()
                        .flat_map(Vec::<Intermediate>::from)
                        .collect(),
                ),
            })]),
            crate::parse::HtmlInElementContext::Element(HtmlElement {
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
                .chain([Intermediate::Literal(">".to_owned(), open_end.span)])
                .chain(
                    children
                        .map(|children| {
                            children
                                .0
                                .into_iter()
                                .flat_map(Vec::<Intermediate>::from)
                                .chain([
                                    Intermediate::Literal("<".to_owned(), children.1.span()),
                                    Intermediate::Literal("/".to_owned(), children.2.span()),
                                    Intermediate::Literal(
                                        children.3.to_string(),
                                        children.3.span(),
                                    ),
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

pub fn simplify(input: Vec<Intermediate>) -> Vec<Intermediate> {
    let (mut acc, current) =
        input
            .into_iter()
            .fold((Vec::new(), None), |(mut acc, current), next| {
                match (current, next) {
                    (None, Intermediate::Literal(lit, span)) => (acc, Some((lit, span))),
                    (Some((lit1, span1)), Intermediate::Literal(lit2, span2)) => (
                        acc,
                        Some((lit1 + &lit2, span1.join(span2).unwrap_or(span1))),
                    ),
                    (Some((lit, span)), Intermediate::For(mut html_for)) => (
                        {
                            acc.push(Intermediate::Literal(lit, span));
                            html_for.body.1 = simplify(html_for.body.1);
                            acc.push(Intermediate::For(html_for));
                            acc
                        },
                        None,
                    ),
                    (Some((lit, span)), Intermediate::While(mut html_while)) => (
                        {
                            acc.push(Intermediate::Literal(lit, span));
                            html_while.body.1 = simplify(html_while.body.1);
                            acc.push(Intermediate::While(html_while));
                            acc
                        },
                        None,
                    ),
                    (Some((lit, span)), Intermediate::If(mut html_if)) => (
                        {
                            acc.push(Intermediate::Literal(lit, span));
                            html_if.then_branch.1 = simplify(html_if.then_branch.1);
                            if let Some(mut else_) = html_if.else_branch {
                                else_.2 = simplify(else_.2);
                                html_if.else_branch = Some(else_);
                            }
                            acc.push(Intermediate::If(html_if));
                            acc
                        },
                        None,
                    ),
                    (Some((lit, span)), Intermediate::ComputedValue(computed)) => (
                        {
                            acc.push(Intermediate::Literal(lit, span));
                            acc.push(Intermediate::ComputedValue(computed));
                            acc
                        },
                        None,
                    ),
                    (Some((lit, span)), Intermediate::Computation(computation)) => (
                        {
                            acc.push(Intermediate::Literal(lit, span));
                            acc.push(Intermediate::Computation(computation));
                            acc
                        },
                        None,
                    ),
                    (None, Intermediate::For(mut html_for)) => (
                        {
                            html_for.body.1 = simplify(html_for.body.1);
                            acc.push(Intermediate::For(html_for));
                            acc
                        },
                        None,
                    ),
                    (None, Intermediate::While(mut html_while)) => (
                        {
                            html_while.body.1 = simplify(html_while.body.1);
                            acc.push(Intermediate::While(html_while));
                            acc
                        },
                        None,
                    ),
                    (None, Intermediate::If(mut html_if)) => (
                        {
                            html_if.then_branch.1 = simplify(html_if.then_branch.1);
                            if let Some(mut else_) = html_if.else_branch {
                                else_.2 = simplify(else_.2);
                                html_if.else_branch = Some(else_);
                            }
                            acc.push(Intermediate::If(html_if));
                            acc
                        },
                        None,
                    ),
                    (None, Intermediate::ComputedValue(value)) => (
                        {
                            acc.push(Intermediate::ComputedValue(value));
                            acc
                        },
                        None,
                    ),
                    (None, Intermediate::Computation(value)) => (
                        {
                            acc.push(Intermediate::Computation(value));
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
