use proc_macro2_diagnostics::Diagnostic;
use syn::{braced, parse::Parse, token::Brace, Block, Expr, LitStr, Pat, Token};

pub struct HtmlChildren {
    pub children: Vec<HtmlChild>,
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

pub enum HtmlChild {
    Literal(LitStr),
    Computed(Block),
    If(HtmlIf),
    For(HtmlForLoop),
}

impl Parse for HtmlChild {
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

pub struct HtmlIf {
    pub if_token: Token![if],
    pub cond: Expr,
    pub then_branch: (Brace, HtmlChildren),
    pub else_branch: Option<(Token![else], Brace, HtmlChildren)>,
}

impl Parse for HtmlIf {
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

pub struct HtmlForLoop {
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: Expr,
    pub body: (Brace, HtmlChildren),
}

impl Parse for HtmlForLoop {
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
