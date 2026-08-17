# del-rs

`del-rs` is WrightKit's Rust implementation of the DeltinScript / OSTW
language. It provides a standalone library and CLI for parsing, loading,
type-checking, and analyzing `.del` / `.ostw` projects with structured
diagnostics and source provenance.

Canonical Workshop catalog data, localization, and emission are provided by
`workshop-rs` rather than duplicated in this repository; the integration
boundary is documented in the [architecture reference](docs/architecture.md).

## Features

- **Recoverable parsing:** retains authored text, comments, trivia, and source
  locations while producing partial trees for invalid or incomplete input.
- **Project loading:** resolves multi-file imports with deterministic ordering.
- **Semantic analysis:** name and type resolution, overloads, access control,
  classes, structs, enums, inheritance, virtual dispatch, generics, lambdas,
  pattern matching, and recursion checks.
- **Typed semantic representation:** preserves allocation/deletion, references,
  dispatch, recursion, lambdas, and storage intent without hard-coding Workshop
  encodings.
- **Tooling APIs:** symbol, reference, type, and resolution queries for Wright
  and other consumers.
- **Compatibility evidence:** a machine-checked support matrix, corpus fixtures,
  provenance records, a bounded semantic oracle, and an evidence report
  (`del-rs compatibility --json`).

## .del / .ostw compatibility

Compatibility targets observable DeltinScript / OSTW semantics for the declared
support surface, not upstream compiler internals or output-text identity.
Support claims are backed by the repository corpus and pinned upstream
evidence.[^upstream-reference]

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
| DEL/OSTW → Workshop compilation | ⏳ Not yet | Requires `workshop-rs` integration |
| Workshop → DEL/OSTW reconstruction | ⏳ Not yet | |

> [!NOTE]
> Editor / VS Code parity and several non-core language experiments are outside
> the `del-rs` language contract. The exact boundaries are documented in
> [Limitations](docs/limitations.md).

Exact feature evidence lives in the
[machine-readable support matrix](docs/support-matrix.toml); see the
[compatibility reference](docs/compatibility.md) for methodology and state
meanings.

## Building

Requirements: Rust 1.80+ (edition 2021). The crate has no runtime dependencies
beyond `serde`, `serde_json`, and `toml`.

```sh
cargo build --release
cargo test --all-targets
```

There are no prebuilt distributions yet. Install from source with
`cargo install --path .` if you want the `del-rs` binary on your `PATH`.

## Quick start

```text
del-rs parse <file> [--json]
del-rs check <file-or-dir> [--json]
del-rs hir <file-or-dir> [--json]
del-rs inspect <file> <line>:<col> [--json]
del-rs matrix [--check] [--json]
del-rs compatibility [--json]
```

Exit codes are `0` for success, `1` for diagnosed source errors, `2` for usage
errors, `3` for internal errors, and `4` for I/O errors. With `--json`, stdout
contains one stable JSON document while human-readable diagnostics go to
stderr.

Example:

```sh
del-rs check tests/corpus/highlevel/enum-basic.del
del-rs matrix --check
```

### Library usage

```rust
use del_rs::semantic::provider::NoopProvider;

let mut sources = del_rs::SourceMap::new();
let id = sources.add_file("main.del".into(), source_text);
let out = del_rs::syntax::parse_source(id, &source_text);

let report = del_rs::api::check_path(path, &NoopProvider::new());
for d in &report.diagnostics {
    println!("[{}] {}", d.code, d.message);
}

let symbol = del_rs::api::symbol_at(&report.semantic, id, offset);
let ty = del_rs::api::type_at(&report.semantic, id, offset);
let matrix = del_rs::matrix::load_and_validate()?;
```

## How it works

```text
.del / .ostw source
    ↓
lexer → recoverable parser
    ↓
project loader
    ↓
semantic analysis
    ↓
typed intermediate representation
    ↓
bounded semantic checks
    ↓
[ integration boundary → workshop-rs ]
```

The [architecture reference](docs/architecture.md) covers module layout,
diagnostics, the Workshop-name integration seam, semantic representation,
public APIs, and test strategy.

## Documentation

Architecture, compatibility, interfaces, provenance, limitations, and
maintainer references are indexed in [`docs/README.md`](docs/README.md).

## Contributing

Contributions are welcome through issues and pull requests. Implementation
sequencing stays in GitHub; durable repository documentation lives under
`docs/`. Run `cargo test --all-targets` before submitting changes and follow
the workspace and repository `AGENTS.md` guidance.

## License

`del-rs` is distributed under the [MIT license](https://opensource.org/licenses/MIT).
Compatibility fixtures retain their recorded upstream provenance and licensing.

[^upstream-reference]: The pinned upstream identity, fixture provenance, and
    licensing rules are recorded in [Provenance](docs/provenance.md).
