use proc_macro2_diagnostics::Diagnostic;
use syn::{
    braced,
    parse::Parse,
    token::{Brace, Token},
    Block, Expr, Ident, LitStr, Pat, Token,
};

pub struct HtmlChildren {
    pub children: Vec<Html<HtmlChildren>>,
}

impl Parse for HtmlChildren {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let mut children = Vec::new();
        while !input.is_empty() && !(input.peek(Token![<]) && input.peek2(Token![/])) {
            let child_start_span = input.span();
            children.push(input.parse().map_err(|err| {
                Diagnostic::from(err)
                    .span_note(child_start_span, "while parsing child")
                    .span_note(span, "while parsing children")
            })?);
        }
        Ok(Self { children })
    }
}

pub enum Html<Inner: Parse> {
    Literal(LitStr),
    Computed(Block),
    If(HtmlIf<Inner>),
    For(HtmlForLoop<Inner>),
    Element(HtmlElement),
}

impl<Inner: Parse> Parse for Html<Inner> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        if input.peek(LitStr) {
            Ok(Self::Literal(input.parse()?))
        } else if input.peek(Token![if]) {
            Ok(Self::If(input.parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing if")
            })?))
        } else if input.peek(Token![for]) {
            Ok(Self::For(input.parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing for")
            })?))
        } else {
            Ok(Self::Computed(input.parse()?))
        }
    }
}

pub struct HtmlIf<Inner: Parse> {
    pub if_token: Token![if],
    pub cond: Expr,
    pub then_branch: (Brace, Inner),
    pub else_branch: Option<(Token![else], Brace, Inner)>,
}

impl<Inner: Parse> Parse for HtmlIf<Inner> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(HtmlIf {
            if_token: input.parse()?,
            cond: {
                let span = input.span();
                input.call(Expr::parse_without_eager_brace).map_err(|err| {
                    Diagnostic::from(err).span_note(span, "while parsing if condition")
                })?
            },
            then_branch: {
                let content;
                (braced!(content in input), {
                    let then_span = content.span();
                    content.parse().map_err(|err| {
                        Diagnostic::from(err).span_note(then_span, "while parsing then branch")
                    })?
                })
            },
            else_branch: {
                if input.peek(Token![else]) {
                    Some({
                        let content;
                        (input.parse()?, braced!(content in input), {
                            let else_span = content.span();
                            content.parse().map_err(|err| {
                                Diagnostic::from(err)
                                    .span_note(else_span, "while parsing else branch")
                            })?
                        })
                    })
                } else {
                    None
                }
            },
        })
    }
}

pub struct HtmlForLoop<Inner: Parse> {
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: Expr,
    pub body: (Brace, Inner),
}

impl<Inner: Parse> Parse for HtmlForLoop<Inner> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let for_token: Token![for] = input.parse()?;

        let pat = Pat::parse_multi_with_leading_vert(input)?;

        let in_token: Token![in] = input.parse()?;
        let expr: Expr = input.call(Expr::parse_without_eager_brace)?;

        let content;
        let brace_token = braced!(content in input);

        Ok(HtmlForLoop {
            for_token,
            pat,
            in_token,
            expr,
            body: (brace_token, content.parse()?),
        })
    }
}

pub struct HtmlAttributeValue {
    pub children: Vec<Html<HtmlAttributeValue>>,
}

impl Parse for HtmlAttributeValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let mut children = Vec::new();
        while !input.is_empty() && !(input.peek(Token![<]) && input.peek2(Token![/])) {
            let child_start_span = input.span();
            children.push(input.parse().map_err(|err| {
                Diagnostic::from(err)
                    .span_note(child_start_span, "while parsing attribute value part")
                    .span_note(span, "while parsing attribute value")
            })?);
        }
        Ok(Self { children })
    }
}

pub struct HtmlAttribute {
    pub key: Ident,
    pub equals: Token![=],
    pub value: Html<HtmlAttributeValue>,
}

impl Parse for HtmlAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            equals: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct HtmlElement {
    pub open_start: Token![<],
    pub open_tag_name: Ident,
    pub attributes: Vec<HtmlAttribute>,
    pub open_end: Token![>],
    pub children: HtmlChildren,
    pub close_start: (Token![<], Token![/]),
    pub close_tag_name: Ident,
    pub close_end: Token![>],
}

impl Parse for HtmlElement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            open_start: input.parse()?,
            open_tag_name: input.parse()?,
            attributes: {
                let mut attributes = Vec::new();
                while !input.peek(Token![>]) {
                    let attribute_start_span = input.span();
                    attributes.push(input.parse().map_err(|err| {
                        Diagnostic::from(err)
                            .span_note(attribute_start_span, "while parsing attribute")
                    })?);
                }
                attributes
            },
            open_end: input.parse()?,
            children: input.parse()?,
            close_start: (input.parse()?, input.parse()?),
            close_tag_name: input.parse()?,
            close_end: input.parse()?,
        })
    }
}
