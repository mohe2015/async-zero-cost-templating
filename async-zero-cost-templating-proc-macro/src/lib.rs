use async_zero_cost_templating_proc_macro2::{
    codegen::{top_level},
    intermediate::{simplify, Intermediate},
    parse::{top_level_parse},
};
use quote::quote;

use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt as _, util::SubscriberInitExt,
};

#[proc_macro]
pub fn html_proc_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tracing_subscriber::registry()
        .with(LevelFilter::TRACE)
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::ACTIVE),
        )
        .init();

    let (html_children, diagnostics) = top_level_parse(input.into());
    let intermediate = Vec::<Intermediate>::from(html_children);
    let intermediate = simplify(intermediate);

    let output = top_level(intermediate);
    let output = quote! {
        #diagnostics
        #output
    };
    error!("{:?}", output.to_string());
    output.into()
}
