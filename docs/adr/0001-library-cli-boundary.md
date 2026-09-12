# ADR-0001: Public library and CLI package boundary

- Status: Accepted
- Date: 2026-09-12
- Related: [Issue #80](https://github.com/wrightkit/deltin-rs/issues/80), [PR #81](https://github.com/wrightkit/deltin-rs/pull/81), [`docs/cli.md`](../cli.md)

## Context

`deltin-rs` needs to serve Rust consumers that parse, load, analyze, inspect,
and compile DEL/OSTW projects. CLI argument parsing, completion, and terminal
presentation are a separate distribution concern. Keeping both in one package
would make ordinary library consumers inherit CLI dependencies and would make
the executable a second owner of language behavior.

Issue #80 established this boundary. PR #81 completed the package split and
kept the CLI as a consumer of the library's public APIs.

Historical evidence: [PR #81](https://github.com/wrightkit/deltin-rs/pull/81)
and its package-split commits, including the follow-up that deferred the
compiler surface until its end-to-end contract was ready.

## Decision

`deltin-rs` is the public Rust embedding crate for DEL/OSTW source, project,
semantic, HIR, tooling, and supported compiler APIs. `deltin-rs-cli` is a thin
executable package that owns command parsing, completion, presentation, and
CLI-only dependencies. The CLI must consume the library's public contracts
rather than reconstructing parsing, semantic, or compiler behavior.

Advanced canonical Workshop interoperability remains an explicit library
boundary for consumers that need it. Canonical Workshop semantics remain owned
by `workshop-rs`; this ADR does not duplicate that model or hide it behind a
CLI-specific integration layer.

## Consequences

Rust embedding does not require CLI dependencies or CLI objects. The executable
can preserve its user-facing command surface while the library evolves as the
language owner. New compiler capabilities are exposed by the library only when
their own contract and evidence are ready; this ADR does not claim completion of
the broader compiler work.

## Compatibility impact

The package split preserves the existing CLI behavior and the library's
language/provenance contracts. Package metadata and dependency boundaries are
part of the distribution contract; command names and language semantics require
their own compatibility evidence when changed.

## Open questions

Future library compiler or reconstruction APIs require their own approved
contracts. The remaining DEL consumer migration and provenance acceptance are
tracked separately in [deltin-rs #107](https://github.com/wrightkit/deltin-rs/issues/107).
