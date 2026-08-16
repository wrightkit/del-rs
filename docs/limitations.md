# del-rs known limitations and gap classification

Status of the Workshop-independent frontend at the completion of issues
#2–#7. Gaps are classified as **lowering-dependent** (Workshop encoding owned
by the workshop-rs integration, issue #8) or **intentionally unsupported**
(editor-only or outside the language contract).

## Lowering-dependent (owned by #8 / workshop-rs)

- Concrete Workshop emission: actions, values, events, variable slots,
  helper rules, dispatch tables, recursion stacks, reference layouts,
  optimizer choices. The typed HIR expresses intent only
  (`docs/architecture.md` §15).
- Canonical Workshop catalog data (actions/values/events/constants):
  `del-rs` never vendors it; the `WorkshopProvider` trait is the documented
  seam. The `NoopProvider` treats every Workshop-facing name as
  unresolved-but-legal.
- Vanilla Workshop superset bodies (`rule("...")`, `variables {}`,
  `subroutines {}`, `settings {}`, hooks): parsed as opaque token spans with
  no frontend semantics.
- Lobby-settings / custom-game-settings imports (`.json`, `.lobby`):
  recorded with provenance, not interpreted.
- `ds.toml` keys other than `entry_point`: validated syntactically, never
  interpreted.
- Decompiler, optimizer, emulator, pathfinding tooling: inventory entries,
  not implemented (see matrix `compiler-utility` / `decompiler`).

## Intentionally unsupported

- VS Code / language-server behaviors (completions, semantic tokens,
  codelens, incremental parse, snippets, debugger, element-count UI) —
  matrix `editor` category, `out-of-scope`.
- `abstract` keyword: not in the pinned upstream surface.
- `interface` semantics: no keyword exists upstream; `class B : A, X` extra
  types are parsed and inert.
- Union types (`T | U`): parsed and recorded; assignability/member semantics
  are not enforced (PM decision Q11).
- `Players` type: reserved in the type list, unexercised by the corpus
  (PM decision Q9).
- JSON import expressions (`import("file.json")`): parse-only; semantics
  planned (PM decision Q5).
- Pattern-binding mutation through non-lvalue operands is rejected
  (SM017/SM048) per corpus; binding *alias* semantics are represented in HIR
  but not executed by the oracle beyond value semantics.

## Known approximation areas (evidence-backed)

- Unknown-type rejection: upstream rejects undeclared type names; del-rs
  treats them as external by the provider contract. Two corpus fixtures were
  reclassified `unknown` with rationale (`struct-ref-inline-*`).
- Struct literal `{0}` single-value form: modeled as a single-value struct
  literal per corpus evidence; upstream mechanics differ internally.
- `define` inference, array-member builtin set, and operator tables are
  corpus-driven; the matrix tracks each entry with evidence paths.
- Auto-for classification follows upstream `Loops.cs` (step is an expression
  statement).
- Lambda captures: by-value snapshot semantics implemented; by-reference is
  a documented model extension, not exercised (PM decision Q1).

## Differential testing

`tests/differential.rs` is gated on the `DEL_RS_UPSTREAM_BIN` environment
variable pointing at a pinned upstream build (see `docs/provenance.md`);
CI stays green without it. It compares accept/reject and
diagnostic-presence agreement only — never output-text identity.
