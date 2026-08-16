# del-rs

`del-rs` is WrightKit's Rust implementation of the DeltinScript / OSTW
language. It is a standalone, Workshop-independent library and CLI: it
parses, loads, type-checks, and analyzes `.del` / `.ostw` source with
structured diagnostics and full source provenance, and lowers it into a typed
intermediate representation. No Workshop backend or catalog data is required
to build or run it.

## What del-rs provides

- a lexer and recoverable parser (comments/trivia and authored text retained);
- multi-file project loading with import resolution and deterministic ordering;
- source provenance (`Span` on every node) and structured, stable diagnostics;
- name resolution, type checking, overload resolution, and access control;
- classes / structs / enums, inheritance and virtual dispatch, generics,
  lambdas and captures, pattern matching, and recursion legality;
- a typed intermediate representation with validation, plus a bounded
  semantic oracle;
- library APIs and a standalone CLI (`parse`, `check`, `hir`, `inspect`, `matrix`).

Workshop emission, catalog data, and localization are outside this crate;
they belong to `wrightkit/workshop-rs`. Workshop-facing names in `.del` /
`.ostw` source are accepted but not yet resolved against catalog data; the
extension point for that resolution is documented in
[`docs/architecture.md`](docs/architecture.md).

## Features

- **Workshop-independent.** The full pipeline runs with zero Workshop backend
  or catalog data.
- **Provenance everywhere.** Every CST/AST/HIR node carries a `Span`
  (file + byte range); diagnostics are structured, stable, and
  JSON-serializable.
- **Recoverable parsing.** Invalid or incomplete source produces structured
  diagnostics and partial trees, never a panic.
- **Typed intermediate representation.** Allocation/deletion, reference
  identity, virtual dispatch, recursion, lambdas, and storage intent are
  expressed without Workshop encodings.
- **Bounded semantic oracle.** A small interpreter distinguishes correct from
  incorrect high-level behavior on corpus cases before any backend exists.
- **Machine-checkable compatibility.** The support matrix
  ([`docs/support-matrix.toml`](docs/support-matrix.toml)) is validated on
  every CI run; evidence and provenance are recorded per feature.
- **Tooling APIs.** Symbol/reference/type/resolution queries over the semantic
  program and HIR for Wright and other consumers.

## .del / .ostw compatibility

Compatibility with OSTW/DeltinScript is **observable semantic compatibility
for the declared support surface**, not upstream compiler internals and not
output-text identity. What counts is whether the same `.del` / `.ostw` source
is accepted, what diagnostics it produces, and how it is interpreted.

| Capability | Status | Notes |
| --- | --- | --- |
| Syntax & parsing | ✅ Supported | Lexer and recoverable parser, comments/trivia retained, corpus-backed |
| Projects & imports | ✅ Supported | Multi-file import resolution and module loading; `ds.toml` project files not yet |
| Type checking | ✅ Supported | Scoping, overload resolution, access control |
| Classes, structs & enums | ✅ Supported | Allocation/deletion, copies, enum storage |
| Inheritance, virtual methods & override | ✅ Supported | |
| Generics | ✅ Supported | Generic parameters and binding |
| Lambdas & closures | ✅ Supported | |
| Pattern matching & recursion | ✅ Supported | |
| Embedded Workshop / lobby data | 🟡 Partial | Vanilla Workshop blocks parse; lobby-settings import not yet |
| Workshop builtins | ⏳ Not yet | Requires the `workshop-rs` catalog |
| DEL/OSTW → Workshop compilation | ⏳ Not yet | Requires `workshop-rs` |
| Workshop → DEL/OSTW reconstruction | ⏳ Not yet | |

Editor / VS Code integration, union types, `abstract`, and `interface`
semantics are deliberately outside the del-rs language contract; see
[`docs/limitations.md`](docs/limitations.md).

Per-feature evidence and the machine-readable matrix live in
[`docs/support-matrix.toml`](docs/support-matrix.toml), with state meanings in
[`docs/compatibility.md`](docs/compatibility.md) and the feature inventory in
[`docs/inventory.md`](docs/inventory.md).

## Building

Requirements: Rust 1.80+ (edition 2021). The crate has no runtime dependencies
beyond `serde`, `serde_json`, and `toml`.

```sh
cargo build --release        # builds the del-rs binary + library
cargo test --all-targets     # unit + integration + corpus harness + matrix check
```

There are no prebuilt distributions yet; install from source with
`cargo install --path .` if you want the `del-rs` binary on your `PATH`.

## Quick start

The CLI is the fastest way to exercise the pipeline. Exit codes: `0` success,
`1` errors found, `2` usage error, `3` internal error, `4` I/O error.

```text
del-rs parse <file> [--json]            # lex + parse; diagnostics + AST/token summary
del-rs check <file-or-dir> [--json]     # parse → project → semantic → HIR → validate
del-rs hir <file-or-dir> [--json]       # lower + validate; HIR summary
del-rs inspect <file> <line>:<col> [--json]  # symbol / type / resolution at a position
del-rs matrix [--check] [--json]        # print / validate the embedded support matrix
```

`--json` prints exactly one JSON document to stdout with stable field names;
human-readable diagnostics go to stderr. The CLI never requires a Workshop
backend.

Example:

```sh
del-rs check tests/corpus/highlevel/enum-basic.del
del-rs matrix --check
```

### Library usage

```rust
use del_rs::semantic::provider::NoopProvider;

// Parse a single file (tokens + AST + diagnostics, provenance preserved).
let mut sources = del_rs::SourceMap::new();
let id = sources.add_file("main.del".into(), source_text);
let out = del_rs::syntax::parse_source(id, &source_text);

// Full pipeline for a file or directory: parse → project → semantic → HIR.
let report = del_rs::api::check_path(path, &NoopProvider::new());
for d in &report.diagnostics {
    println!("[{}] {}", d.code, d.message);
}

// Semantic queries over the program.
let symbol = del_rs::api::symbol_at(&report.semantic, id, offset);
let ty = del_rs::api::type_at(&report.semantic, id, offset);

// The embedded compatibility matrix, validated mechanically.
let matrix = del_rs::matrix::load_and_validate()?;
```

## How it works

```text
.del / .ostw source
    ↓
lexer → recoverable parser          tokens + AST + diagnostics (spans retained)
    ↓
project loader                      imports, deterministic order
    ↓
semantic analysis                   symbols, scopes, types, overloads
    ↓
typed intermediate representation   lowering + invariant validation
    ↓
bounded interpreter                 high-level behavior checks
    ↓
[ integration boundary → workshop-rs ]   (not in this crate)
```

Full details, including the module layout, diagnostics contract, the
Workshop-name extension point, the intermediate-representation shape, the
interpreter, the CLI contract, and the test strategy, are in
[`docs/architecture.md`](docs/architecture.md).

## Documentation

All durable documentation is indexed in
[`docs/README.md`](docs/README.md):

- [Architecture](docs/architecture.md): implemented architecture baseline, module layout, data flow.
- [Compatibility contract](docs/compatibility.md): what compatibility means, matrix states, methodology.
- [Support matrix](docs/support-matrix.toml): machine-readable declared surface, validated on CI.
- [Feature inventory](docs/inventory.md): the declared language/compiler surface with upstream evidence.
- [Syntax notes](docs/syntax-notes.md): lexical/grammar observations from the pinned upstream.
- [Limitations](docs/limitations.md): current support boundary and known gaps.
- [Provenance](docs/provenance.md): pinned upstream oracle and licensing guardrails.
- [PM decisions](docs/decisions.md): ratified product decisions (Q1–Q16).

## Contributing

Contributions are welcome. Open issues and pull requests against
[`wrightkit/del-rs`](https://github.com/wrightkit/del-rs). Implementation
sequencing and acceptance criteria live in GitHub issues; durable contracts
live in `docs/`. Before submitting, run the quality gates above
(`cargo test --all-targets`), and keep changes within the owning repository's
responsibility boundary per the workspace-level
[`AGENTS.md`](../AGENTS.md).

## License

`del-rs` is distributed under the [MIT license](https://opensource.org/licenses/MIT)
(see `Cargo.toml`). Compatibility corpus fixtures are imported under the
upstream MIT license with provenance headers; see
[`docs/provenance.md`](docs/provenance.md) for the licensing rules.
