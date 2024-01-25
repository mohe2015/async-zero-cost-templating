use std::{
    collections::btree_map::Values,
    convert::identity,
    fmt::{Debug, Display},
};

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use proc_macro2_diagnostics::{Diagnostic, SpanDiagnosticExt};
use syn::{
    braced,
    buffer::Cursor,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::{Brace, Else, For, If, In},
    Expr, Ident, LitStr, Pat, Token,
};
use tracing::{error, error_span, instrument};

#[instrument]
pub fn top_level_parse(input: TokenStream) -> (HtmlChildren, TokenStream) {
    // this parse will only fail if we didn't fully consume the input, but we catch that error inside
    let result: syn::Result<HtmlTopLevel> = syn::parse2(input);
    match result {
        Ok(ok) => (
            ok.children,
            ok.diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.emit_as_item_tokens())
                .collect(),
        ),
        Err(err) => (
            HtmlChildren {
                children: Vec::new(),
            },
            err.into_compile_error(),
        ),
    }
}

trait MyParse<T> {
    /// We don't want to always abort parsing on failures to get better IDE support and also show more errors directly
    #[instrument(skip(t_mapper, fun))]
    fn my_parse<Q>(
        self,
        t_mapper: impl Fn(T) -> Q,
        fun: impl Fn(Diagnostic) -> Diagnostic,
        mut diagnostics: Vec<Diagnostic>,
    ) -> Result<(Q, Vec<Diagnostic>), Vec<Diagnostic>>
    where
        Self: Sized + Debug,
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

macro_rules! my_parse {
    ($t: ty) => {
        impl MyParse<$t> for ParseStream<'_> {
            #[instrument]
            fn inner_my_parse(self) -> Result<($t, Vec<Diagnostic>), Vec<Diagnostic>>
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
    };
}

my_parse!(LitStr);
my_parse!(Token![if]);
my_parse!(Token![else]);
my_parse!(Token![<]);
my_parse!(Token![/]);
my_parse!(Token![>]);
my_parse!(Token![!]);
my_parse!(Option<Token![!]>);
my_parse!(Token![=]);
my_parse!(Token![in]);
my_parse!(Token![for]);
my_parse!(proc_macro2::Ident);

#[instrument]
pub fn transpose<T: Debug>(
    input: Result<(T, Vec<Diagnostic>), Vec<Diagnostic>>,
) -> (Result<T, ()>, Vec<Diagnostic>) {
    match input {
        Ok((t, diagnostics)) => (Ok(t), diagnostics),
        Err(diagnostics) => (Err(()), diagnostics),
    }
}

#[derive(Debug)]
pub struct HtmlTopLevel {
    pub children: HtmlChildren,
    pub diagnostics: Vec<Diagnostic>, // maybe do this for all?
}

impl Parse for HtmlTopLevel {
    #[instrument]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = error_span!("HtmlTopLevel");
        let span = span.enter();

        let span = input.cursor().token_stream().span();

        let mut diagnostics = Vec::new();

        let mut children = Vec::new();
        while !input.is_empty() {
            let child_start_span = input.cursor().token_stream().span();
            let result;
            (result, diagnostics) = transpose(input.my_parse(
                identity,
                |diagnostic| {
                    diagnostic
                        .span_note(child_start_span, "while parsing top level child")
                        .span_note(span, "while parsing top level children")
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
        Ok(HtmlTopLevel {
            children: HtmlChildren { children },
            diagnostics,
        })
    }
}

#[derive(Debug)]
pub struct HtmlChildren {
    pub children: Vec<Html<HtmlChildren>>,
}

impl MyParse<HtmlChildren> for ParseStream<'_> {
    #[instrument]
    fn inner_my_parse(self) -> Result<(HtmlChildren, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = self.cursor().token_stream().span();

        let mut diagnostics = Vec::new();

        let mut children = Vec::new();
        while !self.is_empty() && !(self.peek(Token![<]) && self.peek2(Token![/])) {
            let child_start_span = self.cursor().token_stream().span();
            let result;
            // TODO FIXME when erroring I think this could make no progress and then we would have an infinite loop. so the inner function needs to consume?
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

#[derive(Debug)]
pub enum Html<Inner: Debug> {
    Literal(LitStr),
    Computed((Brace, TokenStream)),
    If(HtmlIf<Inner>),
    For(HtmlForLoop<Inner>),
    Element(HtmlElement),
}

impl<Inner: Debug> MyParse<Html<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    #[instrument]
    fn inner_my_parse(self) -> Result<(Html<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
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
            let then_span = self.cursor().token_stream().span();
            if let Ok((brace, content)) = (|| {
                let content;
                Ok((braced!(content in self), content))
            })() {
                Ok((
                    Html::<Inner>::Computed((brace, content.cursor().token_stream())),
                    diagnostics,
                ))
            } else {
                diagnostics.push(then_span.error("expected { }"));
                return Err(diagnostics);
            }
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

#[derive(Debug)]
pub struct HtmlIf<Inner> {
    pub if_token: If,
    pub cond: TokenStream,
    pub then_branch: (Brace, Inner),
    pub else_branch: Option<(Else, Brace, Inner)>,
}

impl<Inner> MyParse<HtmlIf<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    #[instrument]
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
                    if let Ok((brace, content)) = (|| {
                        let content;
                        Ok((braced!(content in self), content))
                    })() {
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

                        if let Ok((brace, content)) = (|| {
                            let content;
                            Ok((braced!(content in self), content))
                        })() {
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

#[derive(Debug)]
pub struct HtmlForLoop<Inner> {
    pub for_token: For,
    pub pat: TokenStream,
    pub in_token: In,
    pub expr: TokenStream,
    pub body: (Brace, Inner),
}

impl<Inner> MyParse<HtmlForLoop<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    #[instrument]
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
        if let Ok((brace_token, content)) = (|| {
            let content;
            Ok((braced!(content in self), content))
        })() {
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

#[derive(Debug)]
pub struct HtmlAttributeValue {
    pub children: Vec<Html<HtmlAttributeValue>>,
}

impl MyParse<HtmlAttributeValue> for ParseStream<'_> {
    #[instrument]
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

#[derive(Debug)]
pub struct HtmlAttribute {
    pub key: Ident,
    pub value: Option<(Token![=], Html<HtmlAttributeValue>)>,
}

impl MyParse<HtmlAttribute> for ParseStream<'_> {
    #[instrument]
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

#[derive(Debug)]
pub struct HtmlTag {
    pub exclamation: Option<Token![!]>,
    pub name: Ident,
}

impl HtmlTag {
    #[instrument]
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
    #[instrument]
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

#[derive(Debug)]
pub struct HtmlElement {
    pub open_start: Token![<],
    pub open_tag_name: HtmlTag,
    pub attributes: Vec<HtmlAttribute>,
    pub open_end: Token![>],
    pub children: Option<(HtmlChildren, Token![<], Token![/], HtmlTag, Token![>])>,
}

impl MyParse<HtmlElement> for ParseStream<'_> {
    #[instrument]
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
