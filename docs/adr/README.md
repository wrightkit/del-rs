# Architecture Decision Records

ADRs preserve point-in-time architecture decisions and their rationale. They
are history, not a database of current implementation reality.

Current durable contracts are routed from
[`../architecture/README.md`](../architecture/README.md). Source, tests, Cargo
metadata, support evidence, and integrations establish current reality.

## Conventions

- ADR numbers are unique, zero-padded, and never reused for a different
  decision.
- `Proposed` means the decision was recorded but not yet accepted.
- `Accepted` means the decision was approved at that point in project history.
- `Superseded` decisions remain in place and link to the replacing decision or
  current contract.
- Backfilled ADRs use the date they are documented, and identify the historical
  Issues, PRs, commits, or documents from which their rationale was recovered.
- Current versions, support counts, Issue progress, migration state, and other
  mutable reality do not belong in ADR status prose.
- An ADR records a durable choice, not every implementation detail or completed
  work item.

## Index

- [ADR-0001: Public library and CLI package boundary](0001-library-cli-boundary.md)
- [ADR-0002: DEL/OSTW core semantics and feature locality](0002-language-core-semantics.md)
- [ADR-0003: DEL HIR and canonical Workshop ownership](0003-hir-workshop-boundary.md)
- [ADR-0004: Domain-local semantic checker responsibilities](0004-semantic-checker-locality.md)

## Backfill classification

| Classification | Material | Treatment |
| --- | --- | --- |
| Backfilled ADR | #80 public embedding/package boundary | ADR-0001 records the library/CLI ownership decision. |
| Backfilled ADR | #2–#7 and #89 language-core decisions | ADR-0002 records the stable core-scope, typed-behavior, declarative-facts, and locality decisions. |
| Backfilled ADR | #6, #29, #30, #91 HIR and Workshop boundary | ADR-0003 records source-owned semantics and DEL-owned lowering into canonical Workshop. |
| Backfilled ADR | #90/#92 semantic checker extraction | ADR-0004 records responsibility locality; it does not freeze file decomposition. |
| Historical-log-covered / non-ADR detail | Q1–Q16 and bounded #31 lowering slices in [`decisions.md`](../decisions.md) | Retain the historical pointer and Git provenance; do not create one ADR per language rule or lowering slice. |
| Externally owned / completed | Canonical Workshop `Program` API and public operations, accepted in [workshop-rs #179](https://github.com/wrightkit/workshop-rs/issues/179) and delivered by [PR #185](https://github.com/wrightkit/workshop-rs/pull/185) | This is an accepted `workshop-rs` decision; its DEL consumer consequences remain tracked separately. |
| Externally owned | Canonical Workshop catalog, WIR, settings, localization, validation, and emission | These decisions belong to `workshop-rs`. |
| Unresolved / externally coordinated | Remaining DEL consumer migration and provenance acceptance in [#107](https://github.com/wrightkit/deltin-rs/issues/107); DEL runtime ABI in [#104](https://github.com/wrightkit/deltin-rs/issues/104) | No accepted ADR is backfilled until the remaining owner contract and evidence are complete. |

The current architecture documents remain the authority for present invariants.
An ADR does not claim that every capability mentioned in its historical source
is implemented today.
