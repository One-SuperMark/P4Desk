# P4Desk JPEG preview patch

This is the fixed jpeg-decoder 0.3.2 crate from crates.io. `SOURCES.json` records
the archive checksum and original source-file hashes. The upstream MIT and
Apache 2.0 licenses and README are retained.

`default-features = false` disables rayon but upstream still starts a pthread
for each color component through its MPSC worker. Its default thread stacks
and unbounded row queues do not meet the firmware's memory bounds.

For `target_os = "espidf"`, `Decoder::select_worker` therefore selects the
existing immediate worker. JPEG preview runs entirely on P4Desk's file worker.
Ordinary host behavior is unchanged. `force-immediate` is an opt-in host-test
feature for the embedded branch. Keep rayon disabled in the application.

The local manifest omits upstream benchmark and development dependencies;
`Cargo.toml.orig` retains the original metadata. It also registers the legacy
`asmjs` cfg value for modern Rust's cfg checking. The source decoder contains
a focused worker-selection test.

Run its library tests with:

```sh
cargo test --manifest-path third_party/jpeg-decoder/Cargo.toml --lib --no-default-features --features force-immediate
```

The two upstream documentation examples open a `tests/reftest/images` JPEG
fixture that is not included in the crates.io archive. They are not executable
fixture tests in this vendored subset; library tests and the application's
self-generated JPEG decoding fixture cover the embedded preview path.
