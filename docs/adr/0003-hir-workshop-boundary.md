# ADR-0003: DEL HIR and canonical Workshop ownership

- Status: Accepted
- Date: 2026-09-12
- Related: [Issue #6](https://github.com/wrightkit/deltin-rs/issues/6), [#29](https://github.com/wrightkit/deltin-rs/issues/29), [#30](https://github.com/wrightkit/deltin-rs/issues/30), [#91](https://github.com/wrightkit/deltin-rs/issues/91), [#107](https://github.com/wrightkit/deltin-rs/issues/107), [workshop-rs #179](https://github.com/wrightkit/workshop-rs/issues/179), [`workshop-boundary.md`](../architecture/workshop-boundary.md)

## Context

DEL/OSTW runtime concepts such as allocation, references, dispatch, recursion,
captures, storage intent, and source provenance need a typed representation
before target lowering. Canonical Workshop representation has a different
owner and lifecycle. The original HIR contract in #6 and the provider/lowering
work in #29/#30/#91 established a boundary that the current architecture
documents preserve.

Historical evidence: the [typed HIR contract in #6](https://github.com/wrightkit/deltin-rs/issues/6),
the [provider/lowering boundary in #30](https://github.com/wrightkit/deltin-rs/issues/30),
the [binding-preparation split in #91](https://github.com/wrightkit/deltin-rs/issues/91),
and the externally owned [Program boundary in workshop-rs PR #185](https://github.com/wrightkit/workshop-rs/pull/185).

## Decision

`deltin-rs` owns DEL/OSTW syntax, semantic meaning, typed HIR, runtime policy,
and the lowering decisions needed to preserve that meaning. HIR expresses
source-language intent and provenance; it does not encode Workshop slots,
helper-rule layouts, target storage schemes, dispatch tables, or reference bit
patterns.

`deltin-rs` lowers that intent through the public canonical Workshop boundary.
`workshop-rs` owns canonical Workshop identities, WIR, settings/localization,
validation, and emission. DEL-specific runtime layouts and compiler helper
strategies remain in `deltin-rs`. A missing capability is fixed in
`workshop-rs` only when it is independently a Workshop concept; otherwise DEL
must lower its source behavior using existing canonical primitives or report an
explicit unsupported boundary.

Workshop-independent parsing, project, semantic, HIR, diagnostic, and tooling
paths do not require complete backend lowering. Workshop reconstruction follows
the reverse ownership direction: canonical Workshop parsing and semantics are
provided by `workshop-rs`, while DEL/OSTW source reconstruction belongs here.

## Consequences

Backend strategies can change without changing source semantics or HIR merely
to match a target encoding. Consumers can distinguish DEL semantic evidence
from canonical Workshop validation/emission evidence. The dependency direction
remains `deltin-rs → workshop-rs`, with no local duplicate catalog, WIR, emitter,
or provider-specific semantic model.

## Compatibility impact

The compatibility target is observable DEL/OSTW meaning and validated canonical
Workshop behavior, not byte-identical Workshop text or upstream helper shape.
Source spans and structured diagnostics remain part of the boundary where the
canonical API supports them. Lowering-dependent and Workshop-independent
support states must remain separately reported.

## Open questions

The canonical public `Program` boundary is an externally owned, completed
decision recorded in [workshop-rs #179](https://github.com/wrightkit/workshop-rs/issues/179)
and delivered by [PR #185](https://github.com/wrightkit/workshop-rs/pull/185).
Remaining DEL consumer migration and provenance acceptance are tracked by
[deltin-rs #107](https://github.com/wrightkit/deltin-rs/issues/107), while the
DEL runtime ABI remains unresolved in [#104](https://github.com/wrightkit/deltin-rs/issues/104).
Those DEL changes must not be represented as accepted history here until their
consumer evidence is complete.
