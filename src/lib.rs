macro_rules! html {
    ($($tt: tt)*) => {};
}

// it should emit blocks of a specified size to reduce fragmentation. This means the goal is not always lowest latency but little overhead and then lowest latency
// syntax inspired by https://yew.rs/docs/concepts/basic-web-technologies/html

type TemplatePart = ();

fn main(title: TemplatePart, inner: TemplatePart) {
    html! {
        <html>
            <head>
                <title>{title}</title>
            </head>
            <body>
                partial!(inner)
            </body>
        </html>
    }
}

html! {
    template!(main("test",
        html! {
            <div class=["hi "{ test }]>
                {
                    let test = get_version();
                }
                "hi what :-)"
                {
                    let result = fetch_database_row().await;
                }
                // maybe accept normal syntax but just in a really specific form
                foreach! (result, |row| {
                    <li>
                        { row.name }
                    </li>
                })
            </div>
        }
    ))
}
