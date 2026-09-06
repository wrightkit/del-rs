# DEL / OSTW / Workshop Boundary

This document defines the current boundary between the DEL/OSTW implementation and canonical Workshop.

## Dependency direction

```text
DEL / OSTW source
    ↓
deltin-rs parsing / project / semantic implementation
    ↓
deltin-rs runtime + compiler lowering
    ↓
workshop-rs canonical WIR / validation / emission
    ↓
Workshop text
```

For reconstruction:

```text
Workshop text
    ↓
workshop-rs parser / canonical WIR
    ↓
deltin-rs reconstruction
    ↓
DEL / OSTW source
```

The durable Rust dependency direction is `deltin-rs → workshop-rs`; `workshop-rs` does not depend back on DEL/OSTW semantics.

## Runtime and lowering ownership

High-level DEL/OSTW concepts such as object/reference lifetime, classes, virtual dispatch, closures/captures, recursion, storage intent, project semantics, and source-language control/type behavior remain `deltin-rs` responsibilities even when their compiled representation uses Workshop primitives.

`workshop-rs` owns canonical Workshop identities, WIR, raw Workshop validation, settings/localization, and emission. It does not own DEL runtime layouts or compiler helper strategies.

A missing canonical Workshop primitive is fixed in `workshop-rs` only when the requirement is independently a Workshop concept. Otherwise `deltin-rs` must lower the source behavior using canonical primitives or report an explicit unsupported boundary.

## HIR boundary

DEL HIR represents source-language semantic intent, not a frozen Workshop encoding. Backend strategies may evolve without changing HIR when the observable DEL semantics and public contracts remain stable.

Do not push Workshop slots, helper-rule shapes, array layouts, temporary names, or other backend encodings upward into source semantics solely because one lowering strategy currently uses them.

## Tooling path

Parsing, project loading, type/semantic checking, inspect/query, and source-aware tooling should remain usable without full Workshop emission when the requested operation does not depend on target Workshop semantics.

## Wright/provider integration

Wright and any LPP provider are downstream integration roles. They do not own DEL/OSTW syntax, type/runtime semantics, or compiler lowering, and must not create a parallel DEL implementation to compensate for an owner gap.