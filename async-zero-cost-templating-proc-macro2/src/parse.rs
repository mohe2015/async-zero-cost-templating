use std::fmt::Display;

use proc_macro2::TokenTree;
use proc_macro2_diagnostics::Diagnostic;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    Expr, Ident, LitStr, Pat, Token,
};

trait MyParse<T> {
    /// We don't want to always abort parsing on failures to get better IDE support and also show more errors directly
    fn my_parse(self) -> Result<(T, Vec<Diagnostic>), Vec<Diagnostic>>;
}

impl<T: Parse> MyParse<T> for ParseStream<'_> {
    fn my_parse(self) -> Result<(T, Vec<Diagnostic>), Vec<Diagnostic>>
    where
        Self: Sized,
    {
        let result = self.parse();
        match result {
            Ok(t) => Ok((t, Vec::new())),
            Err(err) => Err(Vec::from([Diagnostic::from(err)])),
        }
    }
}

trait MyParseExt {
    fn diagnostic_context(self, fun: impl Fn(Diagnostic) -> Diagnostic) -> Self
    where
        Self: Sized;
}

impl<T> MyParseExt for Result<(T, Vec<Diagnostic>), Vec<Diagnostic>> {
    fn diagnostic_context(self, fun: impl Fn(Diagnostic) -> Diagnostic) -> Self
    where
        Self: Sized,
    {
        match self {
            Ok((t, diagnostics)) => Ok((t, diagnostics.into_iter().map(fun).collect())),
            Err(diagnostics) => Err(diagnostics.into_iter().map(fun).collect()),
        }
    }
}

trait MyParseExt2<T> {
    fn append_diagnostics(self, diagnostics: &mut Vec<Diagnostic>) -> T
    where
        Self: Sized;
}

impl<T> MyParseExt2<T> for (T, Vec<Diagnostic>) {
    fn append_diagnostics(self, diagnostics: &mut Vec<Diagnostic>) -> T
    where
        Self: Sized,
    {
        diagnostics.extend(self.1);
        self.0
    }
}

// https://docs.rs/syn/latest/syn/spanned/index.html sounds like nightly should produce much better spans

// self and no errors
// self and errors
// no self and errors

pub struct HtmlChildren {
    pub children: Vec<Html<HtmlChildren>>,
}

impl MyParse<HtmlChildren> for ParseStream<'_> {
    fn my_parse(self) -> Result<(HtmlChildren, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = self.cursor().token_stream().span();

        let mut diagnostics = Vec::new();

        let mut children = Vec::new();
        while !self.is_empty() && !(self.peek(Token![<]) && self.peek2(Token![/])) {
            let child_start_span = self.cursor().token_stream().span();
            let (child, new_diagnostics) = self.my_parse().diagnostic_context(|diagnostic| {
                diagnostic
                    .span_note(child_start_span, "while parsing child")
                    .span_note(span, "while parsing children")
            });
            diagnostics.extend(new_diagnostics);
            if let Ok(child) = child {
                children.push(child);
            }
        }
        (Ok(HtmlChildren { children }), diagnostics)
    }
}

pub enum Html<Inner> {
    Literal(LitStr),
    Computed(TokenTree),
    If(HtmlIf<Inner>),
    For(HtmlForLoop<Inner>),
    Element(HtmlElement),
}

impl<Inner> MyParse<Html<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    fn my_parse(self) -> Result<(Html<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let diagnostics = Vec::new();
        let lookahead = self.lookahead1();
        let span = self.cursor().token_stream().span();
        if lookahead.peek(LitStr) {
            Ok((
                Html::<Inner>::Literal(
                    MyParse::<LitStr>::my_parse(self)?.append_diagnostics(&mut diagnostics),
                ),
                diagnostics,
            ))
        } else if lookahead.peek(Token![if]) {
            Ok(Html::<Inner>::If(self.my_parse().diagnostic_context(
                |diagnostic| diagnostic.span_note(span, "while parsing if"),
            )?))
        } else if lookahead.peek(Token![for]) {
            Ok(Html::<Inner>::For(self.my_parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing for")
            })?))
        } else if lookahead.peek(Brace) {
            Ok(Html::<Inner>::Computed(self.my_parse().map_err(|err| {
                Diagnostic::from(err).span_note(span, "while parsing computed")
            })?))
        } else if lookahead.peek(Token![<]) {
            Ok(Html::<Inner>::Element(self.my_parse().map_err(|err| {
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

impl<Inner> MyParse<HtmlIf<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    fn my_parse(self) -> Result<(HtmlIf<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        Ok(HtmlIf {
            if_token: self.parse()?,
            cond: {
                let span = self.cursor().token_stream().span();
                self.call(Expr::parse_without_eager_brace).map_err(|err| {
                    Diagnostic::from(err).span_note(span, "while parsing if condition")
                })?
            },
            then_branch: {
                let content;
                (braced!(content in self), {
                    let then_span = content.cursor().token_stream().span();
                    content.parse().map_err(|err| {
                        Diagnostic::from(err).span_note(then_span, "while parsing then branch")
                    })?
                })
            },
            else_branch: {
                if self.peek(Token![else]) {
                    Some({
                        let content;
                        (self.parse()?, braced!(content in self), {
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

impl<Inner> MyParse<HtmlForLoop<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    fn my_parse(self) -> Result<(HtmlForLoop<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let for_token: Token![for] = self.parse()?;

        let pat = Pat::parse_multi_with_leading_vert(self)?;

        let in_token: Token![in] = self.parse()?;
        let expr: Expr = self.call(Expr::parse_without_eager_brace)?;

        let content;
        let brace_token = braced!(content in self);

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

impl MyParse<HtmlAttributeValue> for ParseStream<'_> {
    fn my_parse(self) -> Result<(HtmlAttributeValue, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = self.cursor().token_stream().span();

        let mut children = Vec::new();
        while !self.is_empty() && !(self.peek(Token![<]) && self.peek2(Token![/])) {
            let child_start_span = self.cursor().token_stream().span();
            children.push(self.parse().map_err(|err| {
                Diagnostic::from(err)
                    .span_note(child_start_span, "while parsing attribute value part")
                    .span_note(span, "while parsing attribute value")
            })?);
        }
        Ok(HtmlAttributeValue { children })
    }
}

pub struct HtmlAttribute {
    pub key: Ident,
    pub value: Option<(Token![=], Html<HtmlAttributeValue>)>,
}

impl MyParse<HtmlAttribute> for ParseStream<'_> {
    fn my_parse(self) -> Result<(HtmlAttribute, Vec<Diagnostic>), Vec<Diagnostic>> {
        Ok(HtmlAttribute {
            key: self.parse()?,
            value: {
                if self.peek(Token![=]) {
                    Some((self.parse()?, self.parse()?))
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

impl MyParse<HtmlTag> for ParseStream<'_> {
    fn my_parse(self) -> Result<(HtmlTag, Vec<Diagnostic>), Vec<Diagnostic>> {
        Ok(HtmlTag {
            exclamation: self.parse()?,
            name: self.parse()?,
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

impl MyParse<HtmlElement> for ParseStream<'_> {
    fn my_parse(self) -> Result<(HtmlElement, Vec<Diagnostic>), Vec<Diagnostic>> {
        let diagnostics = Vec::new();

        let open_start = self.my_parse()?;
        let open_tag_name: HtmlTag = self.parse()?;
        let open_tag_name_text = open_tag_name.to_string();
        Ok((
            HtmlElement {
                open_start,
                open_tag_name,
                attributes: {
                    let mut attributes = Vec::new();
                    while !self.peek(Token![>]) {
                        let attribute_start_span = self.cursor().token_stream().span();
                        attributes.push(self.parse().map_err(|err| {
                            Diagnostic::from(err)
                                .span_note(attribute_start_span, "while parsing attribute")
                        })?);
                    }
                    attributes
                },
                open_end: self.parse()?,
                children: {
                    if open_tag_name_text != "!doctype" {
                        Some((
                            self.my_parse()?,
                            self.my_parse()?,
                            self.my_parse()?,
                            self.my_parse()?,
                            self.my_parse()?,
                        ))
                    } else {
                        None
                    }
                },
            },
            Vec::new(),
        ))
    }
}
