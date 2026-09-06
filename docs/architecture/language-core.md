# DEL / OSTW Language Core Contract

`deltin-rs` is an independently usable Rust implementation of the DeltinScript / OSTW language. This contract defines semantic ownership and implementation direction; it does not claim every core feature is already implemented.

## Upstream core is the executable specification

For the declared DEL/OSTW core-language surface, the established DeltinScript / OSTW implementation is the executable specification. Core behavior is presumptively in scope unless explicitly excluded as editor/integration functionality or demonstrated to be a non-contractual implementation artifact.

The compatibility inventory, support matrix, corpus, reference probes, and real projects verify implementation completeness and observable compatibility. They do not decide feature-by-feature whether established core language behavior belongs in `deltin-rs`.

Upstream architecture is not a mandate. Understand the source-language behavior and implement it directly in clear Rust rather than mechanically translating upstream internals.

## Semantic ownership

`deltin-rs` owns:

- lexical/source syntax and trivia needed for diagnostics/tooling;
- project loading and import/module semantics;
- name/type/member/overload/access resolution;
- classes, structs, enums, inheritance, virtual dispatch, generics, lambdas, references, recursion, storage and other DEL/OSTW runtime semantics;
- diagnostics and source provenance;
- typed DEL HIR and source-aware tooling semantics;
- DEL/OSTW-specific runtime/compiler lowering;
- Workshop→DEL/OSTW reconstruction.

The Workshop-independent semantic path must remain useful without requiring complete backend lowering.

## Typed behavior, not an inventory language

Use typed Rust to express source-language behavior and invariants, including type/assignability rules, dispatch, capture/reference semantics, runtime lifetime/storage meaning, overload and argument binding, project resolution, and lowering decisions.

Machine-readable inventory/support data is appropriate for capability identity, evidence links, support state, provenance, and other declarative facts. It is not the semantic specification and must not become an interpreted language that defines source behavior.

A corpus entry or support-matrix row proves evidence/support status; it does not authorize a semantic design or narrow the upstream core scope.

## Feature locality

Source-language behavior should have a discoverable domain home. Parser, semantic checker, HIR lowerer, Workshop lowerer, and generic registries are phase infrastructure, not automatic homes for every future feature.

If implementing a feature would deepen an already mixed responsibility, the smallest bounded extraction needed to keep the changed behavior cohesive is within scope. Unrelated cleanup and speculative abstraction remain out of scope.

## Compatibility target

Target observable semantic compatibility: accepted/rejected programs, project behavior, meaningful diagnostics/provenance, source tooling behavior, high-level runtime semantics, lowering results, and declared reconstruction contracts.

Matching upstream helper identity, optimizer shape, formatting, generated names, internal IR, or compiler architecture is not required unless it changes an observable contract.

`deltin-rs` does not invent a WrightKit-only DEL/OSTW dialect.