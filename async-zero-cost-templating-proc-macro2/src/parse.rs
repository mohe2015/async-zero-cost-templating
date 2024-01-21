use syn::{
    parse::{discouraged::AnyDelimiter, Nothing, Parse},
    punctuated::Punctuated,
    LitStr, Token,
};

struct HtmlChildren {
    children: Vec<HtmlChild>,
}

pub enum HtmlChild {
    Literal(LitStr),
}

impl Parse for HtmlChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::Literal(input.parse()?))
        } else {
            todo!()
        }
    }
}

impl Parse for HtmlChildren {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut children = Vec::new();
        while !(input.peek(Token![<]) && input.peek2(Token![/])) {
            children.push(input.parse()?);
        }
        Ok(Self { children })
    }
}
