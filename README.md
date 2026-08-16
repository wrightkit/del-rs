# del-rs

Workshop-independent OSTW/DeltinScript-compatible frontend for the WrightKit
ecosystem. `del-rs` owns DEL/OSTW lexical analysis, recoverable parsing, the
source model with provenance, project/import loading, semantic analysis, the
typed backend-neutral HIR, structured diagnostics, and tooling APIs.

## Scope

The compatibility surface is declared and tracked in
[`docs/support-matrix.toml`](docs/support-matrix.toml) — a machine-checkable
matrix validated by `cargo test --test matrix` on every CI run. Evidence and
provenance live in [`docs/inventory.md`](docs/inventory.md) and
[`docs/provenance.md`](docs/provenance.md); parser behavior is pinned to the
upstream reference implementation (`ItsDeltin/Overwatch-Script-To-Workshop`,
see `docs/provenance.md`) via the corpus under `tests/corpus/`.

Out of scope for this crate: canonical Workshop catalog data, WIR,
localization, and emission (owned by `wrightkit/workshop-rs`; integration is
issue #8). Workshop-facing names bind through the `WorkshopProvider` trait
(`del_rs::semantic::provider`) instead of vendored catalog data.

## Status

- Issue #2 — compatibility inventory/corpus: done (`docs/`, `tests/corpus/`).
- Issue #3 — source frontend and project model: implemented (lexer, recoverable
  parser, CST/AST with provenance, project loader).
- Issues #4–#7 — semantic analysis, advanced semantics, typed HIR, tooling
  APIs: in progress.

## Usage

```text
del-rs parse <file> [--json]      # lex + parse, diagnostics + AST summary
del-rs matrix [--check] [--json]  # embedded compatibility matrix
```

Library entry points (Wright and other consumers):

```rust
let mut sources = del_rs::SourceMap::new();
let id = sources.add_file("main.del".into(), source_text);
let out = del_rs::syntax::parse_source(id, &source_text); // tokens + AST + diagnostics
```

## Development

```text
cargo test        # unit + integration + corpus harness + matrix check
cargo test --test corpus   # corpus fixture expectations (tests/corpus/**)
cargo test --test matrix   # support-matrix mechanical validation
```

See `docs/architecture.md` for the design record and `docs/roadmap.md` for the
issue/PR plan.
