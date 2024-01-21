use syn::{parse::Parse, Block, LitStr, Token};

pub struct HtmlChildren {
    pub children: Vec<HtmlChild>,
}

pub enum HtmlChild {
    Literal(LitStr),
    Computed(Block),
}

impl Parse for HtmlChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::Literal(input.parse()?))
        } else {
            Ok(Self::Computed(input.parse()?))
        }
    }
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
