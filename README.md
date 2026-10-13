# float-abel

Exact abelian-group sums of `f64`, so an incremental engine can add, retract and re-merge floats and
land the same bits under any order, worker count or architecture.

`f64` addition is not associative, so an engine that keeps a running float answer and applies
changes to it lands different bits under different schedules. This crate holds the sum exactly, as a
canonical abelian group with a ring on top, in three exact tiers and one batch-only lossy tier. The
design, including why the usual adaptive binned accumulator is **not** a group, is in
[`docs/DESIGN.md`](docs/DESIGN.md).

The crate is under construction and not yet published.

## Building

```bash
cargo test                                             # the self dev-dependency turns the test features on
cargo test -p float-abel-noalloc                       # Exact with no features: the compact tier alone
cargo +1.99.0 test --features dbsp --test main dbsp_   # dbsp's toolchain; 1.97 and 1.98 ICE inside it
cargo run --example dd_sum -- -w 3                     # Exact as a differential-dataflow diff: SUM on three workers
cargo fuzz run sum_vs_oracle --target-dir "$CARGO_TARGET_DIR/fuzz" -- -max_total_time=60   # a fuzz target; nightly + cargo-fuzz
./scripts/smoke.sh                                     # commit gate
./scripts/regression.sh                                # push gate
git config core.hooksPath .githooks                    # run the gates from git
```

Keep `CARGO_TARGET_DIR` outside any Spotlight-indexed or cloud-synced tree. Every object cargo writes
into a watched directory is queued for indexing or upload, and on macOS that is what makes a warm
build feel cold. For example:

```bash
export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/float-abel"
```

## License

MIT OR Apache-2.0, at your option.
