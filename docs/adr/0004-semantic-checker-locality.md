# ADR-0004: Domain-local semantic checker responsibilities

- Status: Accepted
- Date: 2026-09-12
- Related: [Issue #90](https://github.com/wrightkit/deltin-rs/issues/90), [PR #92](https://github.com/wrightkit/deltin-rs/pull/92), [`language-core.md`](../architecture/language-core.md)

## Context

The semantic checker had accumulated type resolution, name/member/overload
resolution, expression and statement checking, rule traversal, and feature
rules in one implementation unit. This made source-language behavior harder to
locate even though the semantic state was intentionally shared. Issue #90
resolved the responsibility boundary, and PR #92 implemented it without
changing the checked-program contract.

Historical evidence: [PR #92](https://github.com/wrightkit/deltin-rs/pull/92)
records the extraction and its independent compile-time ablation.

## Decision

Keep one explicit shared `Checker` state object where semantic state must be
shared, while locating behavior by domain:

- `semantic/check/resolution.rs` owns type, name, member, and overload
  resolution;
- `semantic/check/expressions.rs` owns expression typing and lvalue rules;
- `semantic/check/statements.rs` owns statement and local-declaration checking;
- `semantic/check/rules.rs` owns rule and body traversal; and
- `semantic/check.rs` remains the state and phase-orchestration facade.

This is a responsibility contract, not a promise that these exact files or a
fixed module count will never change. It does not authorize a generic visitor,
service hierarchy, global semantic state, or unrelated parser/backend cleanup.

## Consequences

Maintainers can find and extend semantic behavior through its source-language
domain while shared ownership and diagnostics remain explicit. Future changes
can extract a cohesive domain when needed without duplicating checker state or
turning the checker into a generic framework.

## Compatibility impact

The decision changes internal discoverability only. Public semantic/check APIs,
diagnostics, provenance, HIR inputs, and Workshop-independent behavior remain
the compatibility surface and require regression evidence when changed.

## Open questions

Future semantic domains may require a bounded responsibility decision when
their behavior does not fit an existing owner. Such a change should update the
current architecture contract if the ownership invariant changes; ordinary
module movement does not require a new ADR.
