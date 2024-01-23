use std::{collections::btree_map::Values, convert::identity, fmt::Display};

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use proc_macro2_diagnostics::{Diagnostic, SpanDiagnosticExt};
use syn::{
    braced,
    buffer::Cursor,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    Expr, Ident, LitStr, Pat, Token,
};

trait MyParse<T> {
    /// We don't want to always abort parsing on failures to get better IDE support and also show more errors directly
    fn my_parse<Q>(
        self,
        t_mapper: impl Fn(T) -> Q,
        fun: impl Fn(Diagnostic) -> Diagnostic,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<(Q, Vec<Diagnostic>), Vec<Diagnostic>>
    where
        Self: Sized,
    {
        let inner = self.inner_my_parse();
        match inner {
            Ok((value, inner_diagnostics)) => Ok((t_mapper(value), {
                diagnostics.extend(inner_diagnostics.into_iter().map(fun));
                diagnostics
            })),
            Err(inner_diagnostics) => Err({
                diagnostics.extend(inner_diagnostics.into_iter().map(fun));
                diagnostics
            }),
        }
    }

    fn inner_my_parse(self) -> Result<(T, Vec<Diagnostic>), Vec<Diagnostic>>;
}

impl<T: Parse> MyParse<T> for ParseStream<'_> {
    fn inner_my_parse(self) -> Result<(T, Vec<Diagnostic>), Vec<Diagnostic>>
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

pub fn transpose<T>(
    input: Result<(T, Vec<Diagnostic>), Vec<Diagnostic>>,
) -> (Result<T, ()>, Vec<Diagnostic>) {
    match input {
        Ok((t, diagnostics)) => (Ok(t), diagnostics),
        Err(diagnostics) => (Err(()), diagnostics),
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
    fn inner_my_parse(self) -> Result<(HtmlChildren, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = self.cursor().token_stream().span();

        let mut diagnostics = Vec::new();

        let mut children = Vec::new();
        while !self.is_empty() && !(self.peek(Token![<]) && self.peek2(Token![/])) {
            let child_start_span = self.cursor().token_stream().span();
            let result;
            (result, diagnostics) = transpose(self.my_parse(
                identity,
                |diagnostic| {
                    diagnostic
                        .span_note(child_start_span, "while parsing child")
                        .span_note(span, "while parsing children")
                },
                diagnostics,
            ));
            match result {
                Ok(child) => {
                    children.push(child);
                }
                Err(err) => {}
            }
        }
        Ok((HtmlChildren { children }, diagnostics))
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
    fn inner_my_parse(self) -> Result<(Html<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let diagnostics = Vec::new();
        let lookahead = self.lookahead1();
        let span = self.cursor().token_stream().span();
        if lookahead.peek(LitStr) {
            Ok(MyParse::<LitStr>::my_parse(
                self,
                Html::<Inner>::Literal,
                |diagnostic| diagnostic,
                diagnostics,
            )?)
        } else if lookahead.peek(Token![if]) {
            Ok(MyParse::<HtmlIf<Inner>>::my_parse(
                self,
                Html::<Inner>::If,
                |diagnostic| diagnostic.span_note(span, "while parsing if"),
                diagnostics,
            )?)
        } else if lookahead.peek(Token![for]) {
            Ok(MyParse::<HtmlForLoop<Inner>>::my_parse(
                self,
                Html::<Inner>::For,
                |diagnostic| diagnostic.span_note(span, "while parsing for"),
                diagnostics,
            )?)
        } else if lookahead.peek(Brace) {
            Ok(MyParse::<TokenTree>::my_parse(
                self,
                Html::<Inner>::Computed,
                |diagnostic| diagnostic.span_note(span, "while parsing computed"),
                diagnostics,
            )?)
        } else if lookahead.peek(Token![<]) {
            Ok(MyParse::<HtmlElement>::my_parse(
                self,
                Html::<Inner>::Element,
                |diagnostics| diagnostics.span_note(span, "while parsing element"),
                diagnostics,
            )?)
        } else {
            Err(Vec::from([Diagnostic::from(lookahead.error())]))
        }
    }
}

pub struct HtmlIf<Inner> {
    pub if_token: Token![if],
    pub cond: TokenStream,
    pub then_branch: (Brace, Inner),
    pub else_branch: Option<(Token![else], Brace, Inner)>,
}

impl<Inner> MyParse<HtmlIf<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    fn inner_my_parse(self) -> Result<(HtmlIf<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        Ok((
            HtmlIf {
                if_token: {
                    let result;
                    (result, diagnostics) =
                        MyParse::<Token![if]>::my_parse(self, identity, identity, diagnostics)?;
                    result
                },
                cond: {
                    let result = self.step(|cursor| {
                        let mut rest = *cursor;
                        let mut tokens = TokenStream::new();
                        while let Some((tt, next)) = rest.token_tree() {
                            tokens.extend(std::iter::once(rest.token_tree().unwrap().0));
                            match &tt {
                                TokenTree::Group(group)
                                    if group.delimiter() == Delimiter::Brace =>
                                {
                                    return Ok((tokens, rest));
                                }
                                _ => rest = next,
                            }
                        }
                        Err(cursor.error("no { was found after this point"))
                    });
                    match result {
                        Ok(value) => value,
                        Err(error) => {
                            diagnostics.push(error.into());
                            return Err(diagnostics);
                        }
                    }
                },
                then_branch: {
                    let then_span = self.cursor().token_stream().span();
                    let mut content;
                    if let Ok(brace) = (|| Ok(braced!(content in self)))() {
                        let result;
                        (result, diagnostics) = MyParse::<Inner>::my_parse(
                            &content,
                            identity,
                            |diagnostic| {
                                diagnostic.span_note(then_span, "while parsing then branch")
                            },
                            diagnostics,
                        )?;
                        (brace, result)
                    } else {
                        diagnostics.push(then_span.error("expected { }"));
                        return Err(diagnostics);
                    }
                },
                else_branch: {
                    if self.peek(Token![else]) {
                        let else_span = self.cursor().token_stream().span();
                        let else_;
                        (else_, diagnostics) = MyParse::<Token![else]>::my_parse(
                            self,
                            identity,
                            identity,
                            diagnostics,
                        )?;

                        let mut content;
                        if let Ok(brace) = (|| Ok(braced!(content in self)))() {
                            let result;
                            (result, diagnostics) = MyParse::<Inner>::my_parse(
                                &content,
                                identity,
                                |diagnostic| {
                                    diagnostic.span_note(else_span, "while parsing else branch")
                                },
                                diagnostics,
                            )?;
                            Some((else_, brace, result))
                        } else {
                            diagnostics.push(else_span.error("expected { }"));
                            return Err(diagnostics);
                        }
                    } else {
                        None
                    }
                },
            },
            diagnostics,
        ))
    }
}

pub struct HtmlForLoop<Inner> {
    pub for_token: Token![for],
    pub pat: TokenStream,
    pub in_token: Token![in],
    pub expr: TokenStream,
    pub body: (Brace, Inner),
}

impl<Inner> MyParse<HtmlForLoop<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    fn inner_my_parse(self) -> Result<(HtmlForLoop<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let for_token: Token![for];
        (for_token, diagnostics) =
            MyParse::<Token![for]>::my_parse(self, identity, identity, diagnostics)?;

        let result = self.step(|cursor| {
            let mut rest = *cursor;
            let mut tokens = TokenStream::new();
            while let Some((tt, next)) = rest.token_tree() {
                tokens.extend(std::iter::once(rest.token_tree().unwrap().0));
                match &tt {
                    TokenTree::Ident(ident) if ident.to_string() == "in" => {
                        return Ok((tokens, rest));
                    }
                    _ => rest = next,
                }
            }
            Err(cursor.error("no { was found after this point"))
        });
        let pat = match result {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error.into());
                return Err(diagnostics);
            }
        };

        let in_token: Token![in];
        (in_token, diagnostics) =
            MyParse::<Token![in]>::my_parse(self, identity, identity, diagnostics)?;

        let result = self.step(|cursor| {
            let mut rest = *cursor;
            let mut tokens = TokenStream::new();
            while let Some((tt, next)) = rest.token_tree() {
                tokens.extend(std::iter::once(rest.token_tree().unwrap().0));
                match &tt {
                    TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                        return Ok((tokens, rest));
                    }
                    _ => rest = next,
                }
            }
            Err(cursor.error("no { was found after this point"))
        });
        let expr = match result {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error.into());
                return Err(diagnostics);
            }
        };

        let loop_span = self.cursor().token_stream().span();
        let mut content;
        if let Ok(brace_token) = (|| Ok(braced!(content in self)))() {
            let result;
            (result, diagnostics) = MyParse::<Inner>::my_parse(
                &content,
                identity,
                |diagnostic| diagnostic.span_note(loop_span, "while parsing loop body"),
                diagnostics,
            )?;
            Ok((
                HtmlForLoop {
                    for_token,
                    pat,
                    in_token,
                    expr,
                    body: (brace_token, result),
                },
                diagnostics,
            ))
        } else {
            diagnostics.push(loop_span.error("expected { }"));
            return Err(diagnostics);
        }
    }
}

pub struct HtmlAttributeValue {
    pub children: Vec<Html<HtmlAttributeValue>>,
}

impl MyParse<HtmlAttributeValue> for ParseStream<'_> {
    fn inner_my_parse(self) -> Result<(HtmlAttributeValue, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = self.cursor().token_stream().span();

        let mut diagnostics = Vec::new();

        let mut children = Vec::new();
        // TODO FIXME this impl is comletely broken
        while !self.is_empty() && !(self.peek(Token![<]) && self.peek2(Token![/])) {
            let child_start_span = self.cursor().token_stream().span();
            let result;
            (result, diagnostics) = transpose(self.my_parse(
                identity,
                |diagnostic| {
                    diagnostic
                        .span_note(child_start_span, "while parsing attribute value part")
                        .span_note(span, "while parsing attribute value")
                },
                diagnostics,
            ));
            match result {
                Ok(child) => {
                    children.push(child);
                }
                Err(err) => {}
            }
        }
        Ok((HtmlAttributeValue { children }, diagnostics))
    }
}

pub struct HtmlAttribute {
    pub key: Ident,
    pub value: Option<(Token![=], Html<HtmlAttributeValue>)>,
}

impl MyParse<HtmlAttribute> for ParseStream<'_> {
    fn inner_my_parse(self) -> Result<(HtmlAttribute, Vec<Diagnostic>), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        Ok((
            HtmlAttribute {
                key: {
                    let value;
                    (value, diagnostics) =
                        MyParse::my_parse(self, identity, identity, diagnostics)?;
                    value
                },
                value: {
                    if self.peek(Token![=]) {
                        Some((
                            {
                                let value;
                                (value, diagnostics) =
                                    MyParse::my_parse(self, identity, identity, diagnostics)?;
                                value
                            },
                            {
                                let value;
                                (value, diagnostics) =
                                    MyParse::my_parse(self, identity, identity, diagnostics)?;
                                value
                            },
                        ))
                    } else {
                        None
                    }
                },
            },
            diagnostics,
        ))
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
    fn inner_my_parse(self) -> Result<(HtmlTag, Vec<Diagnostic>), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        Ok((
            HtmlTag {
                exclamation: {
                    let value;
                    (value, diagnostics) =
                        MyParse::my_parse(self, identity, identity, diagnostics)?;
                    value
                },
                name: {
                    let value;
                    (value, diagnostics) =
                        MyParse::my_parse(self, identity, identity, diagnostics)?;
                    value
                },
            },
            diagnostics,
        ))
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
    fn inner_my_parse(self) -> Result<(HtmlElement, Vec<Diagnostic>), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        let open_start = {
            let value;
            (value, diagnostics) = MyParse::my_parse(self, identity, identity, diagnostics)?;
            value
        };
        let open_tag_name: HtmlTag = {
            let value;
            (value, diagnostics) = MyParse::my_parse(self, identity, identity, diagnostics)?;
            value
        };
        let open_tag_name_text = open_tag_name.to_string();
        Ok((
            HtmlElement {
                open_start,
                open_tag_name,
                attributes: {
                    let mut attributes = Vec::new();
                    while !self.peek(Token![>]) {
                        let attribute_start_span = self.cursor().token_stream().span();
                        attributes.push({
                            let value;
                            (value, diagnostics) = MyParse::my_parse(
                                self,
                                identity,
                                |diagnostic| {
                                    diagnostic
                                        .span_note(attribute_start_span, "while parsing attribute")
                                },
                                diagnostics,
                            )?;
                            value
                        });
                    }
                    attributes
                },
                open_end: {
                    let value;
                    (value, diagnostics) =
                        MyParse::my_parse(self, identity, identity, diagnostics)?;
                    value
                },
                children: {
                    if open_tag_name_text != "!doctype" {
                        Some((
                            {
                                let value;
                                (value, diagnostics) =
                                    MyParse::my_parse(self, identity, identity, diagnostics)?;
                                value
                            },
                            {
                                let value;
                                (value, diagnostics) =
                                    MyParse::my_parse(self, identity, identity, diagnostics)?;
                                value
                            },
                            {
                                let value;
                                (value, diagnostics) =
                                    MyParse::my_parse(self, identity, identity, diagnostics)?;
                                value
                            },
                            {
                                let value;
                                (value, diagnostics) =
                                    MyParse::my_parse(self, identity, identity, diagnostics)?;
                                value
                            },
                            {
                                let value;
                                (value, diagnostics) =
                                    MyParse::my_parse(self, identity, identity, diagnostics)?;
                                value
                            },
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
