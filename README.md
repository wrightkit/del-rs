# del-rs

`del-rs` is the Workshop-independent OSTW/DeltinScript-compatible frontend of
the [WrightKit](https://github.com/wrightkit) ecosystem. It owns DEL/OSTW
lexical analysis, recoverable parsing, a source model with provenance,
project/import loading, semantic analysis, a typed backend-neutral HIR,
structured diagnostics, and tooling APIs — all in one standalone Rust crate
with no `workshop-rs` dependency.

## Responsibility boundary

`del-rs` owns everything needed to parse, load, resolve, type-check, and lower
`.del` / `.ostw` source into a typed, backend-neutral program:

- lexer and recoverable parser (comments/trivia and authored text retained);
- multi-file project loading with import resolution and deterministic ordering;
- source provenance (`Span` on every node) and structured, stable diagnostics;
- name resolution, type checking, overload resolution, and access control;
- classes / structs / enums, inheritance and virtual dispatch, generics,
  lambdas and captures, pattern matching, and recursion legality;
- typed DEL HIR with validation, plus a bounded semantic oracle;
- library APIs and a standalone CLI (`parse`, `check`, `hir`, `inspect`, `matrix`).

`del-rs` does **not** own canonical Workshop catalog data, WIR, localization,
or emission — those belong to `wrightkit/workshop-rs`. Workshop-facing
names bind through the [`WorkshopProvider`](docs/architecture.md) trait
(`del_rs::semantic::provider`) rather than vendored catalog data.

## Features

- **Workshop-independent.** Full frontend pipeline runs with zero Workshop
  backend or catalog data; `NoopProvider` treats every Workshop-facing name as
  unresolved-but-legal.
- **Provenance everywhere.** Every CST/AST/HIR node carries a `Span`
  (file + byte range); diagnostics are structured, stable, and
  JSON-serializable.
- **Recoverable parsing.** Invalid or incomplete source produces structured
  diagnostics and partial trees, never a panic.
- **Backend-neutral typed HIR.** Allocation/deletion, reference identity,
  virtual dispatch, recursion, lambdas, and storage intent are expressed
  without Workshop encodings.
- **Bounded semantic oracle.** A tree-walking interpreter distinguishes
  correct from incorrect high-level behavior on corpus cases before any
  backend exists.
- **Machine-checkable compatibility.** The support matrix
  ([`docs/support-matrix.toml`](docs/support-matrix.toml)) is validated on
  every CI run; evidence and provenance are recorded per feature.
- **Tooling APIs.** Symbol/reference/type/resolution queries over the semantic
  program and HIR for Wright and other consumers.

## .del / .ostw compatibility

Compatibility means **observable semantic compatibility for the declared
support surface** — not upstream compiler architecture and not output-text
identity. The declared surface is the machine-readable
[`docs/support-matrix.toml`](docs/support-matrix.toml) (128 entries), backed
by the corpus under `tests/corpus/`, the
[`docs/inventory.md`](docs/inventory.md) feature inventory, and the pinned
upstream oracle ([`docs/provenance.md`](docs/provenance.md)). State meanings
are defined in [`docs/compatibility.md`](docs/compatibility.md).

| Capability | Matrix scope | State |
| --- | --- | --- |
| Lexing and recoverable parsing | `syntax.*` (47 entries) | frontend-supported |
| Project / import loading | `project.import-resolution`, `project.modules-resolution` | frontend-supported |
| Source provenance and structured diagnostics | cross-cutting, every phase | implemented |
| Name resolution and type checking | `semantic.*` (19 entries) | semantic-supported |
| Class / struct / enum semantics | `syntax.classes/structs/enums`, `runtime-semantics.struct-copy`, `runtime-semantics.enum-storage` | semantic-supported |
| Inheritance / virtual / override | `syntax.inheritance`, `syntax.virtual-override`, `runtime-semantics.virtual-dispatch` | semantic-supported |
| Generics | `syntax.generics`, `semantic.generic-binding` | semantic-supported |
| Lambdas and captures | `syntax.lambdas`, `semantic.lambda-capture`, `runtime-semantics.lambda-closures` | semantic-supported |
| Pattern matching | `semantic.pattern-matching`, `runtime-semantics.pattern-binding` | semantic-supported |
| Recursion semantics | `runtime-semantics.recursion` | semantic-supported |
| Typed DEL HIR | `hir/` layer | implemented |
| Semantic inspection / tooling APIs | `del_rs::api`, CLI `inspect` | implemented |
| Workshop builtin / catalog binding | `workshop-lowering.workshop-catalog` | lowering-dependent (provider contract) |
| DEL/OSTW → Workshop compilation | `workshop-lowering.*` (18 entries) | lowering-dependent |
| Workshop → DEL/OSTW reconstruction | `decompiler.*` | planned |
| Editor / VS Code parity | `editor.*` (10 entries) | out-of-scope |

Matrix snapshot (source of truth: `docs/support-matrix.toml`): 49 entries
`frontend-supported`, 35 `semantic-supported`, 16 `planned`, 18
`lowering-dependent`, 10 `out-of-scope`. A single aggregate percentage is
deliberately not reported: the matrix includes lowering-dependent and
intentionally out-of-scope capabilities, so one number would misrepresent the
declared support boundary. See [`docs/compatibility.md`](docs/compatibility.md)
for the contract and methodology.

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
backend — the `NoopProvider` is the only provider in this crate.

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
project loader                      imports, ds.toml entry, deterministic order
    ↓
semantic analysis                   symbols, scopes, types, overloads, provider-bound externals
    ↓
typed backend-neutral HIR           lowering + invariant validation
    ↓
semantic oracle                     bounded interpreter (high-level behavior)
    ↓
[ integration boundary → workshop-rs ]   (not in this crate)
```

Full details, including the module layout, diagnostics contract, provider
boundary, HIR shape, oracle semantics, CLI contract, and test strategy, are in
[`docs/architecture.md`](docs/architecture.md).

## Documentation

All durable documentation is indexed in
[`docs/README.md`](docs/README.md):

- [Architecture](docs/architecture.md) — implemented architecture baseline, module layout, data flow.
- [Compatibility contract](docs/compatibility.md) — what compatibility means, matrix states, methodology.
- [Support matrix](docs/support-matrix.toml) — machine-readable declared surface, validated on CI.
- [Feature inventory](docs/inventory.md) — the declared language/compiler surface with upstream evidence.
- [Syntax notes](docs/syntax-notes.md) — lexical/grammar observations from the pinned upstream.
- [Limitations](docs/limitations.md) — current support boundary, lowering-dependent vs unsupported.
- [Provenance](docs/provenance.md) — pinned upstream oracle and licensing guardrails.
- [PM decisions](docs/decisions.md) — ratified product decisions (Q1–Q16).

## Contributing

Contributions are welcome — open issues and pull requests against
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
