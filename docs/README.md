# del-rs Documentation

This directory is the durable documentation surface for `del-rs`. Documents
are organized by durable contract, not by implementation milestone:
implementation sequencing and acceptance criteria live in GitHub issues and
pull requests; what remains here describes the crate as it currently exists
and the boundaries it commits to.

## Documentation model

```text
GitHub issues/PRs            implementation scope, sequencing, acceptance (historical record)
  └─ docs/decisions.md       ratified product decisions (Q1–Q16)
      └─ docs/architecture.md   implemented architecture baseline (living)
          └─ docs/compatibility.md   compatibility contract (living)
              └─ reference docs & evidence   inventory, syntax-notes, matrix, provenance, limitations
```

## Index

### Architecture

- [`architecture.md`](architecture.md) — implemented architecture baseline:
  governing constraints, design decisions (D1–D6), module layout, source
  model, lexer/parser/project/semantic/HIR/oracle design, public API, CLI
  contract, and test strategy. This is the authoritative design record.

### Compatibility

- [`compatibility.md`](compatibility.md) — the human-readable compatibility
  contract: what compatibility means (observable semantics, not output-text
  identity), `.del`/`.ostw` as accepted source forms, support-matrix state
  meanings, the Workshop-independent frontend vs. end-to-end boundary,
  corpus/differential methodology, and the pinned upstream oracle.
- [`support-matrix.toml`](support-matrix.toml) — the machine-readable declared
  support surface (128 entries), validated mechanically on every CI run
  (`tests/matrix.rs` and `del-rs matrix --check`). **Source of truth** for
  what is supported.
- [`inventory.md`](inventory.md) — the declared language/compiler surface
  inventoried from the pinned upstream with per-feature evidence
  (`path@commit`, wiki pages).
- [`syntax-notes.md`](syntax-notes.md) — precise lexical/grammar observations
  from the pinned upstream (comment kinds, token set, keywords, number forms,
  strings, grammar details).
- [`limitations.md`](limitations.md) — evergreen support-boundary document:
  lowering-dependent vs. intentionally unsupported capabilities, and
  evidence-backed approximation areas.
- [`provenance.md`](provenance.md) — pinned upstream oracle identity, license
  guardrails for the corpus, and the re-pinning procedure.
- [`workshop-conformance.md`](workshop-conformance.md) — evidence report
  schema and the integration boundary with canonical `workshop-rs` feature
  identities.

### Interfaces

- CLI contract: commands, flags, exit codes, and the `--json` envelope are
  documented in [`architecture.md`](architecture.md) §18; the binary prints
  its own help via `del-rs --help`.
- Library API: the stable surface (`parse`, project loading, semantic/HIR
  queries, oracle, matrix) is documented in [`architecture.md`](architecture.md)
  §17 and exercised in the `tests/` integration suites.

### Decisions

- [`decisions.md`](decisions.md) — PM ratifications for architecture
  questions Q1–Q16 (2026-08-16): binding product decisions with upstream
  evidence. Where a ratification corrected a claim in `architecture.md`
  (Q3 import extension, Q14 auto-for, Q16 number forms), the correction is
  applied in the architecture text.

### Development and testing

- Test targets (all auto-discovered by `cargo test`):
  `tests/parse.rs` (lexer/parser), `tests/semantic.rs` (semantics),
  `tests/advanced.rs` (advanced semantics), `tests/hir.rs` (HIR lowering +
  validation + oracle), `tests/corpus.rs` (corpus harness + evidence report +
  project fixtures), `tests/matrix.rs` (matrix mechanical validation),
  `tests/cli.rs` (CLI smoke tests).
- Corpus fixtures live under `tests/corpus/` with `// source` / `// license`
  / `// expect` headers. Evidence, gap, and matrix-link directives are
  documented in [`workshop-conformance.md`](workshop-conformance.md).
- CI (`.github/workflows/ci.yml`) runs `cargo build --all-targets`,
  `cargo test --all-targets`, the matrix gate, and the corpus harness.

## Authority

| Contract | Document | Normative scope |
| --- | --- | --- |
| Architecture | [`architecture.md`](architecture.md) | Module responsibilities, data flow, provider/HIR seams. |
| Compatibility | [`compatibility.md`](compatibility.md) | State meanings, methodology, oracle boundary. |
| Declared surface | [`support-matrix.toml`](support-matrix.toml) | Per-capability support states with evidence. |
| Product decisions | [`decisions.md`](decisions.md) | Ratified Q1–Q16 answers (binding). |
| Provenance | [`provenance.md`](provenance.md) | Oracle pin, licensing, re-pinning. |
