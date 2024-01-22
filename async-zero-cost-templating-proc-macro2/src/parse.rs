use std::fmt::Display;

use proc_macro2::TokenTree;
use proc_macro2_diagnostics::Diagnostic;
use syn::{
    braced, parse::{Parse, ParseStream}, spanned::Spanned, token::Brace, Block, Expr, Ident, LitStr, Pat, Token,
};

trait MyParse {
    /// We don't want to always abort parsing on failures to get better IDE support and also show more errors directly
    fn parse(input: ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> where Self: Sized;
}

// https://docs.rs/syn/latest/syn/spanned/index.html sounds like nightly should produce much better spans

// self and no errors
// self and errors
// no self and errors

pub struct HtmlChildren {
    pub children: Vec<Html<HtmlChildren>>,
}

impl MyParse for HtmlChildren {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = input.cursor().token_stream().span();

        let mut children = Vec::new();
        while !input.is_empty() && !(input.peek(Token![<]) && input.peek2(Token![/])) {
            let child_start_span = input.cursor().token_stream().span();
            MyParse::parse(input, diagnostics);
            // we want to add context so we need to know which diagnostics got added
            let child = .map_err(|err| {
                Diagnostic::from(err)
                    .span_note(child_start_span, "while parsing child")
                    .span_note(span, "while parsing children")
            });
            match child {
                Ok(child) => children.push(child),
             Err(error) => error.emit_as_expr_tokens();

            }
        }
        Ok(Self { children })
    }
}

pub enum Html<Inner> {
    Literal(LitStr),
    Computed(TokenTree),
    If(HtmlIf<Inner>),
    For(HtmlForLoop<Inner>),
    Element(HtmlElement),
}

impl<Inner: MyParse> MyParse for Html<Inner> {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
        let lookahead = input.lookahead1();
        let span = input.cursor().token_stream().span();
        if lookahead.peek(LitStr) {
            Ok(Self::Literal(input.parse()?))
        } else if lookahead.peek(Token![if]) {
            Ok(Self::If(input.parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing if")
            })?))
        } else if lookahead.peek(Token![for]) {
            Ok(Self::For(input.parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing for")
            })?))
        } else if lookahead.peek(Brace) {
            Ok(Self::Computed(input.parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing computed")
            })?))
        } else if lookahead.peek(Token![<]) {
            Ok(Self::Element(input.parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing element")
            })?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct HtmlIf<Inner> {
    pub if_token: Token![if],
    pub cond: Vec<TokenTree>,
    pub then_branch: (Brace, Inner),
    pub else_branch: Option<(Token![else], Brace, Inner)>,
}

impl<Inner: MyParse> MyParse for HtmlIf<Inner> {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
        Ok(HtmlIf {
            if_token: input.parse()?,
            cond: {
                let span = input.cursor().token_stream().span();
                input.call(Expr::parse_without_eager_brace).map_err(|err| {
                    Diagnostic::from(err).span_note(span, "while parsing if condition")
                })?
            },
            then_branch: {
                let content;
                (braced!(content in input), {
                    let then_span = content.cursor().token_stream().span();
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
                            let else_span = content.cursor().token_stream().span();
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

pub struct HtmlForLoop<Inner> {
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: Vec<TokenTree>,
    pub body: (Brace, Inner),
}

impl<Inner: MyParse> MyParse for HtmlForLoop<Inner> {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
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

impl MyParse for HtmlAttributeValue {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = input.cursor().token_stream().span();

        let mut children = Vec::new();
        while !input.is_empty() && !(input.peek(Token![<]) && input.peek2(Token![/])) {
            let child_start_span = input.cursor().token_stream().span();
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
    pub value: Option<(Token![=], Html<HtmlAttributeValue>)>,
}

impl MyParse for HtmlAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
        Ok(Self {
            key: input.parse()?,
            value: {
                if input.peek(Token![=]) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
        })
    }
}

pub struct HtmlTag {
    pub exclamation: Option<Token![!]>,
    pub name: Ident,
}

impl HtmlTag {
    pub fn span(&self) -> proc_macro2::Span {
        if let Some(exclamation) = self.exclamation {
            exclamation.span().join(self.name.span()).unwrap()
        } else {
            self.name.span()
        }
    }
}

impl Display for HtmlTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if self.exclamation.is_some() {
                "!".to_owned()
            } else {
                String::new()
            },
            self.name.to_string()
        )
    }
}

impl MyParse for HtmlTag {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
        Ok(Self {
            exclamation: input.parse()?,
            name: input.parse()?,
        })
    }
}

pub struct HtmlElement {
    pub open_start: Token![<],
    pub open_tag_name: HtmlTag,
    pub attributes: Vec<HtmlAttribute>,
    pub open_end: Token![>],
    pub children: Option<(HtmlChildren, Token![<], Token![/], HtmlTag, Token![>])>,
}

impl MyParse for HtmlElement {
    fn parse(input: syn::parse::ParseStream) -> Result<(Self, Vec<Diagnostic>), Vec<Diagnostic>> {
        let open_start = input.parse()?;
        let open_tag_name: HtmlTag = input.parse()?;
        let open_tag_name_text = open_tag_name.to_string();
        Ok(Self {
            open_start,
            open_tag_name,
            attributes: {
                let mut attributes = Vec::new();
                while !input.peek(Token![>]) {
                    let attribute_start_span = input.cursor().token_stream().span();
                    attributes.push(input.parse().map_err(|err| {
                        Diagnostic::from(err)
                            .span_note(attribute_start_span, "while parsing attribute")
                    })?);
                }
                attributes
            },
            open_end: input.parse()?,
            children: {
                if open_tag_name_text != "!doctype" {
                    Some((
                        input.parse()?,
                        input.parse()?,
                        input.parse()?,
                        input.parse()?,
                        input.parse()?,
                    ))
                } else {
                    None
                }
            },
        })
    }
}
