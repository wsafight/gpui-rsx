# Parser and expansion fuzzing

Install `cargo-fuzz` (`cargo install cargo-fuzz`) and a nightly Rust toolchain, then
run from the repository root:

```sh
cargo +nightly fuzz run expand -- -max_total_time=60 -max_len=4096
```

The target exercises token parsing, class diagnostics and macro expansion in
permissive, strict and Tailwind modes. Invalid syntax should produce errors;
panics are failures. Commit minimized reproductions as regular regression tests.
The workspace's stable `robustness` tests provide a deterministic smoke test too.
