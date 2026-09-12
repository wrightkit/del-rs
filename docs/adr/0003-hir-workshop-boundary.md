# ADR-0003: DEL HIR and canonical Workshop ownership

- Status: Accepted
- Date: 2026-09-12
- Related: [Issue #6](https://github.com/wrightkit/deltin-rs/issues/6), [#29](https://github.com/wrightkit/deltin-rs/issues/29), [#30](https://github.com/wrightkit/deltin-rs/issues/30), [#91](https://github.com/wrightkit/deltin-rs/issues/91), [#107](https://github.com/wrightkit/deltin-rs/issues/107), [workshop-rs ADR-0008](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0008-canonical-public-program-boundary.md), [workshop-rs #179](https://github.com/wrightkit/workshop-rs/issues/179), [`workshop-boundary.md`](../architecture/workshop-boundary.md)

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
and the [canonical public Program boundary in workshop-rs ADR-0008](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0008-canonical-public-program-boundary.md),
with [PR #185](https://github.com/wrightkit/workshop-rs/pull/185) as historical implementation evidence.

## Decision

`deltin-rs` owns DEL/OSTW syntax, semantic meaning, typed HIR, runtime policy,
and the lowering decisions needed to preserve that meaning. HIR expresses
source-language intent and provenance; it does not encode Workshop slots,
helper-rule layouts, target storage schemes, dispatch tables, or reference bit
patterns.

`deltin-rs` lowers that intent through `workshop_rs::Program`, the canonical
public Workshop boundary defined by [workshop-rs ADR-0008](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0008-canonical-public-program-boundary.md):

```text
typed DEL HIR
    ↓
DEL/OSTW runtime + compiler lowering
    ↓
canonical public workshop_rs::Program
    ↓
Workshop validation / emission
```

`workshop-rs` owns the public Workshop concepts, canonical identities,
settings/localization, validation, and emission. Its internal/support WIR and
storage representation is not the public DEL-facing contract. DEL-specific
runtime layouts and compiler helper strategies remain in `deltin-rs`. A missing
capability is fixed in `workshop-rs` only when it is independently a Workshop
concept; otherwise DEL must lower its source behavior through the canonical
`Program` boundary or report an explicit unsupported boundary.

Workshop-independent parsing, project, semantic, HIR, diagnostic, and tooling
paths do not require complete backend lowering. Workshop reconstruction follows
the reverse ownership direction: canonical Workshop parsing and semantics are
provided by `workshop-rs`, while DEL/OSTW source reconstruction belongs here.

## Consequences

Backend strategies can change without changing source semantics or HIR merely
to match a target encoding. Consumers can distinguish DEL semantic evidence
from canonical Workshop validation/emission evidence. The dependency direction
remains `deltin-rs → workshop-rs`, with no local duplicate catalog, public
`Program`, emitter, or provider-specific semantic model.

## Compatibility impact

The compatibility target is observable DEL/OSTW meaning and validated canonical
Workshop behavior, not byte-identical Workshop text or upstream helper shape.
Source spans and structured diagnostics remain part of the boundary where the
canonical API supports them. Lowering-dependent and Workshop-independent
support states must remain separately reported.

## Boundary exclusions

The canonical public `Program` boundary is defined by
[workshop-rs ADR-0008](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0008-canonical-public-program-boundary.md).
This ADR does not decide the DEL runtime ABI or a complete provenance/source-
attachment contract; either would require a separate material decision if
adopted. Issues [#104](https://github.com/wrightkit/deltin-rs/issues/104) and
[#107](https://github.com/wrightkit/deltin-rs/issues/107) are historical
evidence for those decision boundaries, not status records.
