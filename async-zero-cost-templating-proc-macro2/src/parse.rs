use syn::{braced, parse::Parse, token::Brace, Block, Expr, LitStr, Token};

pub struct HtmlChildren {
    pub children: Vec<HtmlChild>,
}

impl Parse for HtmlChildren {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut children = Vec::new();
        while !input.is_empty() && !(input.peek(Token![<]) && input.peek2(Token![/])) {
            children.push(input.parse()?);
        }
        Ok(Self { children })
    }
}

pub enum HtmlChild {
    Literal(LitStr),
    Computed(Block),
    If(HtmlIf),
}

impl Parse for HtmlChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::Literal(input.parse()?))
        } else if input.peek(Token![if]) {
            Ok(Self::If(input.parse()?))
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
        let content;
        Ok(HtmlIf {
            if_token: input.parse()?,
            cond: input.parse()?,
            then_branch: (braced!(content in input), content.parse()?),
            else_branch: {
                if input.peek(Token![else]) {
                    Some({
                        let content;
                        (input.parse()?, braced!(content in input), content.parse()?)
                    })
                } else {
                    None
                }
            },
        })
    }
}
