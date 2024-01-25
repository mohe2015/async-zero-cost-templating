# async-zero-cost-templating

```
RUSTFLAGS="-Zproc-macro-backtrace" cargo +nightly test

# Rust does parser recovery so the output of this is not equal to the macro output
cargo expand -p async-zero-cost-templating --test variable
cargo rustc --package async-zero-cost-templating --test variable --profile=check -- -Zunpretty=expanded
```

Use rust-analyzer's expand macro feature to find out why it is broken