use quote::quote;
use std::{
    convert::identity,
    fmt::{Debug, Display},
};

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use proc_macro2_diagnostics::{Diagnostic, SpanDiagnosticExt};
use syn::{
    braced, bracketed, ext::IdentExt, parse::{Parse, ParseStream}, spanned::Spanned, token::{Brace, Bracket, Else, For, If, In}, Ident, LitStr, Token
};
use tracing::instrument;
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt as _, util::SubscriberInitExt,
};

use crate::{
    codegen::top_level,
    intermediate::{simplify, Intermediate},
};

#[instrument(ret)]
pub fn top_level_parse(input: TokenStream) -> TokenStream {
    let _ = tracing_subscriber::registry()
        .with(LevelFilter::OFF)
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::ACTIVE),
        )
        .try_init();

    // this parse will only fail if we didn't fully consume the input
    // if this crashes then you probably didn't directly consume these but just extracted them which doesn't work
    let html_top_level: HtmlTopLevel = match syn::parse2(input) {
        Ok(ok) => ok,
        Err(err) => return Diagnostic::from(err).error("this is a serde internal error, likely some nested method did read this but not actually consume it?").emit_as_expr_tokens(),
    };
    let diagnostics = html_top_level
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.emit_as_expr_tokens());

    let intermediate = Vec::<Intermediate>::from(html_top_level.children);
    let intermediate = simplify(intermediate);

    let output = top_level(intermediate);
    let output = quote! {
        {
            #(#diagnostics)*
            #output
        }
    };
    error!("{:?}", output.to_string());

    output
}

trait MyParse<T> {
    /// We don't want to always abort parsing on failures to get better IDE support and also show more errors directly
    #[instrument(err(Debug), ret, name = "MyParse", skip(t_mapper, fun))]
    fn my_parse<Q: Debug>(
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
impl MyParse<Ident> for ParseStream<'_> {
    fn inner_my_parse(self) -> Result<(Ident, Vec<Diagnostic>), Vec<Diagnostic>>
    where
        Self: Sized,
    {
        let result = Ident::parse_any(self);
        match result {
            Ok(t) => Ok((t, Vec::new())),
            Err(err) => Err(Vec::from([Diagnostic::from(err)])),
        }
    }
}

#[instrument(ret)]
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
    pub diagnostics: Vec<Diagnostic>, // TODO FIXME put this somewhere else so its not used inside if and for
}

impl From<HtmlTopLevel> for Vec<Intermediate> {
    fn from(value: HtmlTopLevel) -> Self {
        value.children.into()
    }
}

impl Parse for HtmlTopLevel {
    #[instrument(err(Debug), ret, name = "HtmlTopLevel")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
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
                Err(_err) => {}
            }
        }
        error!("{:?}", input);
        Ok(HtmlTopLevel {
            children: HtmlChildren { children },
            diagnostics,
        })
    }
}


impl MyParse<HtmlTopLevel> for ParseStream<'_> {
    #[instrument(err(Debug), ret, name = "HtmlTopLevel")]
    fn inner_my_parse(self) -> Result<(HtmlTopLevel, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = self.cursor().token_stream().span();

        let mut diagnostics = Vec::new();

        let mut children = Vec::new();
        while !self.is_empty() {
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
                Err(_err) => {}
            }
        }
        Ok((HtmlTopLevel { children: HtmlChildren { children }, diagnostics: Vec::new() }, diagnostics))
    }
}


#[derive(Debug)]
pub struct HtmlChildren {
    pub children: Vec<Html<HtmlTopLevel>>,
}

impl MyParse<HtmlChildren> for ParseStream<'_> {
    #[instrument(err(Debug), ret, name = "HtmlChildren")]
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
                Err(_err) => {}
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
    #[instrument(err(Debug), ret, name = "Html<Inner>")]
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
                    Html::<Inner>::Computed((brace, content.parse().unwrap())),
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
            self.step(|cursor| {
                if let Some((_, next)) = cursor.token_tree() {
                    Ok(((), next))
                } else {
                    Ok(((), *cursor))
                }
            })
            .unwrap();
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

impl<Inner: Debug> MyParse<HtmlIf<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    #[instrument(err(Debug), ret, name = "HtmlIf<Inner>")]
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
                            match &tt {
                                TokenTree::Group(group)
                                    if group.delimiter() == Delimiter::Brace =>
                                {
                                    return Ok((tokens, rest));
                                }
                                _ => {
                                    tokens.extend(std::iter::once(rest.token_tree().unwrap().0));
                                    rest = next;
                                }
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

impl<Inner: Debug> MyParse<HtmlForLoop<Inner>> for ParseStream<'_>
where
    for<'a> ParseStream<'a>: MyParse<Inner>,
{
    #[instrument(err(Debug), ret, name = "HtmlForLoop<Inner>")]
    fn inner_my_parse(self) -> Result<(HtmlForLoop<Inner>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let for_token: Token![for];
        (for_token, diagnostics) =
            MyParse::<Token![for]>::my_parse(self, identity, identity, diagnostics)?;

        let result = self.step(|cursor| {
            let mut rest = *cursor;
            let mut tokens = TokenStream::new();
            while let Some((tt, next)) = rest.token_tree() {
                match &tt {
                    TokenTree::Ident(ident) if *ident == "in" => {
                        return Ok((tokens, rest));
                    }
                    _ => {
                        tokens.extend(std::iter::once(rest.token_tree().unwrap().0));
                        rest = next;
                    }
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
                match &tt {
                    TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                        return Ok((tokens, rest));
                    }
                    _ => {
                        tokens.extend(std::iter::once(rest.token_tree().unwrap().0));
                        rest = next;
                    }
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
    #[instrument(err(Debug), ret, name = "HtmlAttributeValue")]
    fn inner_my_parse(self) -> Result<(HtmlAttributeValue, Vec<Diagnostic>), Vec<Diagnostic>> {
        let span = self.cursor().token_stream().span();

        let mut diagnostics = Vec::new();

        let mut children = Vec::new();
        while !self.is_empty() {
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
                Err(_err) => {}
            }
        }
        Ok((HtmlAttributeValue { children }, diagnostics))
    }
}

#[derive(Debug)]
pub struct HtmlAttribute {
    pub key: Ident,
    pub value: Option<(Token![=], HtmlAttributeValue)>,
}

impl MyParse<HtmlAttribute> for ParseStream<'_> {
    #[instrument(err(Debug), ret, name = "HtmlAttribute")]
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
                        // TODO FIXME check for string or []
                        let eq: Token![=];
                        (eq, diagnostics) =
                            MyParse::my_parse(self, identity, identity, diagnostics)?;
                        let lookahead1 = self.lookahead1();

                        let value;
                        (value, diagnostics) = if lookahead1.peek(LitStr) {
                            MyParse::<LitStr>::my_parse(
                                self,
                                |value| HtmlAttributeValue {
                                    children: Vec::from([Html::<HtmlAttributeValue>::Literal(
                                        value,
                                    )]),
                                },
                                identity,
                                diagnostics,
                            )?
                        } else if lookahead1.peek(Bracket) {
                            let then_span = self.cursor().token_stream().span();
                            if let Ok((_bracket, content)) = (|| {
                                let content;
                                Ok((bracketed!(content in self), content))
                            })() {
                                MyParse::<HtmlAttributeValue>::my_parse(
                                    &content,
                                    identity,
                                    identity,
                                    diagnostics,
                                )?
                            } else {
                                diagnostics.push(then_span.error("expected { }"));
                                return Err(diagnostics);
                            }
                        } else {
                            diagnostics.push(Diagnostic::from(lookahead1.error()));
                            return Err(diagnostics);
                        };
                        Some((eq, value))
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
    #[instrument(ret, name = "HtmlTag::span")]
    pub fn span(&self) -> proc_macro2::Span {
        if let Some(exclamation) = self.exclamation {
            exclamation
                .span()
                .join(self.name.span())
                .unwrap_or_else(|| self.name.span())
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
            self.name
        )
    }
}

impl MyParse<HtmlTag> for ParseStream<'_> {
    #[instrument(err(Debug), ret, name = "HtmlTag")]
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
    #[instrument(err(Debug), ret, name = "HtmlElement")]
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
        let attributes = {
            let mut attributes = Vec::new();
            while !self.peek(Token![>]) {
                let attribute_start_span = self.cursor().token_stream().span();
                attributes.push({
                    let value;
                    (value, diagnostics) = MyParse::my_parse(
                        self,
                        identity,
                        |diagnostic| {
                            diagnostic.span_note(attribute_start_span, "while parsing attribute")
                        },
                        diagnostics,
                    )?;
                    value
                });
            }
            attributes
        };
        let open_end = {
            let value;
            (value, diagnostics) = MyParse::my_parse(self, identity, identity, diagnostics)?;
            value
        };
        let children = {
            if open_tag_name_text != "!doctype"
                && open_tag_name_text != "meta"
                && open_tag_name_text != "link"
            {
                Some((
                    {
                        let value;
                        (value, diagnostics) =
                            MyParse::my_parse(self, identity, identity, diagnostics)?;
                        value
                    },
                    {
                        let value;
                        (value, diagnostics) = MyParse::my_parse(
                            self,
                            identity,
                            |diagnostic| {
                                diagnostic.help(format!("maybe {open_tag_name_text} is supposed to be a self-closing tag but the template library doesn't know that?"))
                            },
                            diagnostics,
                        )?;
                        value
                    },
                    {
                        let value;
                        (value, diagnostics) =
                            MyParse::my_parse(self, identity, identity, diagnostics)?;
                        value
                    },
                    {
                        let close_tag_name: HtmlTag;
                        (close_tag_name, diagnostics) =
                            MyParse::my_parse(self, identity, identity, diagnostics)?;
                        let close_tag_name_text = close_tag_name.to_string();
                        if open_tag_name_text != close_tag_name.to_string() {
                            diagnostics.push(open_tag_name.span()
                            .error(format!("mismatched tag {open_tag_name_text}"))
                            .span_error(close_tag_name.span(), format!("{} not matching {}", open_tag_name_text, close_tag_name_text))
                            .help(format!("maybe {open_tag_name_text} is supposed to be a self-closing tag but the template library doesn't know that?")))
                        }
                        close_tag_name
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
        };
        Ok((
            HtmlElement {
                open_start,
                open_tag_name,
                attributes,
                open_end,
                children,
            },
            diagnostics,
        ))
    }
}
