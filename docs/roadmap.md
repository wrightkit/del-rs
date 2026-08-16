# del-rs Roadmap — Workshop-Independent Frontend (#2–#7)

Status: **active tracking document** · Owner: Product Management · Scope: issues #2–#7 of
`wrightkit/del-rs`. Issues #1 (parent/vision), #8 (integration — blocked on #7 +
`wrightkit/workshop-rs#2` public contracts), and #9 (decompilation — blocked on #8) are
**out of scope** and remain untouched.

Compatibility contract (from #1): `DEL/OSTW source -> source model -> DEL semantic model ->
typed DEL HIR -> [integration boundary] -> workshop-rs`. Compatibility means observable
semantic compatibility for the declared support surface, not output-text identity.

---

## 1. Issue → Deliverables Map

Each issue is delivered as exactly one branch + one Draft PR (see §4). Artifacts listed are the
concrete things that satisfy the acceptance criteria (quoted below from each issue).

### #2 — Feature inventory and compatibility corpus
PR: `feat/2-inventory-and-corpus`. Artifacts:

- `docs/provenance.md` — pinned upstream OSTW/DeltinScript reference commit/version used as the
  initial compatibility oracle; per-fixture provenance records.
- `docs/inventory.md` — user-visible language/compiler surface inventoried from upstream source,
  tests, examples, bundled modules, docs, and representative real-world projects, classified per
  capability class (syntax, semantic analysis, high-level runtime semantics,
  Workshop-dependent lowering, compiler utility, decompiler, editor integration, out of scope).
- `docs/support-matrix.toml` — machine-readable support matrix with explicit states
  (`planned`, `frontend-supported`, `semantic-supported`, `lowering-dependent`,
  `end-to-end-supported`) and a `VS Code`/editor-only vs `del-rs` requirement column.
- `docs/architecture.md` — compatibility contract, upstream-quirk vs intended-semantics split,
  and the `workshop-rs` integration boundary.
- `corpus/` — provenance-aware fixture tree: upstream parser/semantic/high-level tests plus
  representative real-world cases, each with provenance; a manifest tying fixtures to matrix
  entries.
- Differential-test harness (runner + minimal result diffing, executing the pinned reference
  compiler where useful) — lives in this PR, is exercised by later PRs.

Acceptance criteria this satisfies:

- "A **mechanically checkable** support matrix covers the declared DEL/OSTW language and
  compiler surface."
- "Every tracked feature has **evidence/provenance** and an **explicit compatibility state**."
- "The corpus includes upstream parser/semantic/high-level tests and **representative
  real-world cases** where provenance permits."
- "The matrix clearly **separates VS Code/editor-only behavior** from `del-rs` compatibility
  requirements."
- "Workshop-dependent cases are **explicitly identified** rather than blocking frontend work."
- "Differential testing can expose observable semantic gaps **without requiring output-text
  identity**."

### #3 — Frontend bootstrap: lexer, CST, parser, project model
PR: `feat/3-frontend-bootstrap`. Artifacts:

- Crate/workspace bootstrap, CI workflow, repo-local guidance (CONTRIBUTING/AGENTS notes),
  license + provenance guardrails (fixture-import policy from #2).
- Lexer for the declared syntax surface.
- CST/source model retaining comments, trivia, authored identifiers, source ranges, file
  provenance.
- Recoverable parser + AST for declarations, expressions, statements, rules, modifiers, types,
  imports, project/file structure; structured diagnostics; partial trees for invalid input.
- Project loader with import resolution, deterministic provenance, cycle/error handling — no
  Workshop backend.
- Positive/negative parser fixtures tied to #2 (`docs/syntax-notes.md` documents the
  implemented surface).
- Thin CLI skeleton (parse/check stub) to make the frontend exercisable from day one.

Acceptance criteria:

- "The repository **builds and tests as a standalone Rust project**."
- "Declared frontend-supported DEL/OSTW syntax **parses into documented CST/AST structures with
  stable source spans**."
- "Invalid or incomplete source produces **structured diagnostics and useful partial syntax
  trees** where practical."
- "Multi-file/imported projects can be loaded with **deterministic provenance and cycle/error
  handling** according to corpus evidence."
- "Comments/trivia and authored identifiers are **retained** sufficiently for diagnostics and
  source tooling."
- "**No `workshop-rs` dependency** is required to parse or load DEL/OSTW projects."

### #4 — Core semantic and type system
PR: `feat/4-semantic-core`. Artifacts:

- Semantic model: symbols, lexical/project scopes, declaration binding, access control, imports,
  references, semantic identities.
- Inventory-backed type system: primitives/source types, arrays/collections, function types,
  conversions, operators, member access, calls, overload resolution, optional/default arguments.
- Semantic rules: variables, functions, methods, constructors, rules, control flow, constants,
  assignments, storage/source modifiers.
- Workshop intrinsic **provider boundary** (external namespace contract — no catalog data).
- Structured semantic diagnostics with stable source provenance; positive/negative fixtures for
  every supported construct.

Acceptance criteria:

- "Parsed projects **resolve into a documented semantic program** independent of Workshop
  emission."
- "Supported declarations, references, scopes, conversions, calls, operators, and access rules
  have **positive and negative fixtures**."
- "Semantic diagnostics **preserve source spans and cross-file provenance**."
- "Workshop-facing names can remain **externally bound/unresolved through a documented provider
  contract** rather than copied catalog data."
- "Relevant support-matrix entries **advance to `semantic-supported`** where no Workshop
  lowering is required."

### #5 — Advanced language semantics
PR: `feat/5-advanced-semantics`. Artifacts:

- Class/struct/enum, inheritance, virtual/override/abstract, constructor, member semantics.
- Generics/type parameters and compatibility constraints.
- Lambda/function-value semantics, closure/capture.
- Pattern matching and union/type-pattern behavior (per corpus).
- Recursion legality and semantic behavior independent of eventual Workshop encoding.
- Cross-feature interaction coverage: arrays/collections × inheritance × generics × overload
  resolution × immutable/value semantics.
- Positive/negative fixtures derived from upstream semantic/high-level tests and representative
  projects (from #2 corpus).

Acceptance criteria:

- "Inventory-backed class/struct/enum, inheritance/override, generic, lambda, pattern-matching,
  and recursion semantics **resolve and diagnose compatibly** within the declared frontend
  surface."
- "Cross-feature interaction cases are covered by **corpus-backed tests rather than only
  isolated syntax tests**."
- "High-level semantic identities and relationships are **inspectable through the semantic
  model**."
- "Features requiring concrete runtime encoding are **clearly represented for later HIR/backend
  work without leaking Workshop implementation details** into the frontend."
- "Relevant support-matrix entries **advance to `semantic-supported`** where backend execution
  is not required."

### #6 — Typed DEL HIR and abstract runtime semantics
PR: `feat/6-typed-hir`. Artifacts:

- Typed HIR definition (backend-neutral): allocation/deallocation, object/reference identity,
  field/member access, virtual dispatch, function/lambda invocation, recursion, control flow,
  storage intent.
- Lowering from the #5 semantic program into HIR, preserving source provenance end to end.
- `new`/`delete`, invalid/stale reference semantics, generation/lifetime intent,
  value/reference distinctions modeled without concrete Workshop encodings.
- HIR invariants/validation pass with source provenance on violations.
- Minimal semantic oracle: an execution/test abstraction sufficient to validate
  backend-neutral behavior — explicitly NOT a second Workshop runtime.
- Workshop intrinsics referenced externally via the #4 provider contract.

Acceptance criteria:

- "Advanced DEL semantic constructs **lower into a documented typed HIR without requiring
  `workshop-rs`**."
- "HIR **expresses the observable intent** needed for allocation/deletion, reference lifetime,
  virtual dispatch, recursion, lambdas, and storage semantics while remaining independent of
  Workshop encoding."
- "HIR **validation catches invalid internal states with source provenance**."
- "Corpus-backed semantic tests can **distinguish correct/incorrect high-level behavior**
  before backend integration where practical."
- "The later Workshop adapter can consume this HIR **without redesigning the parser or
  semantic model**."

### #7 — Frontend completeness, public APIs, CLI, final QA
PR: `feat/7-completeness-and-apis`. Artifacts:

- Completeness drive: every non-Workshop-dependent matrix entry reaches `frontend-supported` or
  `semantic-supported` with corpus evidence and tests.
- Stabilized structured diagnostics across parse, project loading, semantic analysis, advanced
  semantics, HIR validation (machine-consumable, source-attributed).
- Public library API surface: parsing, project loading, symbol/reference lookup, type/semantic
  queries, HIR inspection, source provenance, compatibility/support metadata.
- Standalone CLI for frontend validation/inspection (no Workshop emission).
- Representative multi-file corpus projects validated end-to-end through semantic model + HIR.
- Known-limitations documentation; explicit `lowering-dependent` vs unsupported classification.

Acceptance criteria:

- "**All declared non-Workshop-dependent compatibility items have evidence-backed status and
  tests**."
- "Consumers can **parse/check/inspect** DEL/OSTW projects through **documented library/CLI
  APIs without invoking a Workshop backend**."
- "Diagnostics are **structured, source-attributed, and stable enough for machine consumers**."
- "Cross-file symbols/references, types, high-level semantic relationships, and **HIR
  provenance are queryable** where supported."
- "Representative corpus projects **reach a resolved semantic/HIR program or produce expected
  diagnostics**."
- "Remaining gaps are explicitly classified as **`lowering-dependent` or unsupported with
  rationale**."

---

## 2. Dependency Graph

```
#2 inventory + corpus (evidence base)
   │ initial syntax inventory │ corpus fixtures │ harness │ matrix refinement (continuous)
   ▼                          ▼                 ▼         ▼
#3 frontend bootstrap ──► #4 core semantics ──► #5 advanced semantics ──► #6 typed HIR
   (lexer→CST→parser→project)    │                  │                        │
        │                        │                  │                        │
        └── CLI skeleton ────────┴──────────────────┴────────────────────────┴──► #7
   #7 = pre-integration readiness gate                                        completeness + APIs + CLI + QA
        │
        ▼
   #8 Workshop integration (BLOCKED: #7 ready + wrightkit/workshop-rs#2 contracts)
        ▼
   #9 decompilation (BLOCKED: #8 stable)
```

Hard edges (must be sequential):

- `#2 initial syntax inventory → #3 parser scope` — parser cannot target syntax that is not
  inventoried.
- `#3 → #4` — semantic layer consumes the source/project model.
- `#4 → #5` — advanced semantics build on core types/scopes/overload resolution.
- `#5 → #6` — HIR lowering consumes the resolved semantic program incl. advanced features.
- `#6 → #7` — HIR inspection API and end-to-end validation need the HIR.
- `#7 → #8` — #8 explicitly blocked until #7 and `workshop-rs#2` contracts exist.

Parallelizable (see §3 for windows): #2 evidence work vs #3 crate/CI bootstrap; #2 harness vs
#3 parser; #2 fixture collection for #4/#5 during #3; #7 QA tooling design during #5/#6; #6
oracle design during #5; CLI skeleton (early) vs semantic implementation.

---

## 3. Execution Order + Parallelization Windows

Sequential pipeline with fork/merge points. Step N can start when its stated prerequisite
commit is merged (or, where marked *fork*, on a side branch without blocking the main line).

| # | Step | Starts when | Done when |
|---|------|-------------|-----------|
| 1 | Bootstrap: workspace/CI/guardrails (part of #3) | immediately | CI green on empty scaffold |
| 2 | #2: pin upstream ref, inventory, matrix v1, provenance docs | after 1 | matrix covers declared surface, all entries evidence-backed |
| 3 | #3: lexer → CST → recoverable parser → project model + fixtures | initial #2 syntax inventory exists | all #3 gates pass |
| 4 | #4: semantic core + provider boundary + diagnostics + fixtures | #3 merged | all #4 gates pass |
| 5 | #5: classes/inheritance, generics, lambdas, patterns, recursion + interactions | #4 merged | all #5 gates pass |
| 6 | #6: typed HIR + lowering + validation + minimal oracle | #5 merged | all #6 gates pass |
| 7 | #7: API stabilization, CLI completion, matrix completeness, docs, final QA | #6 merged | all #7 gates pass; DoD (§8) |

Parallelization windows:

- **W1 (steps 1–2):** #2 evidence base ∥ #3 crate/CI/governance bootstrap. Bootstrap needs no
  inventory; inventory needs no scaffold.
- **W2 (steps 2–3):** differential-test harness (runner + diffing) written while the parser is
  implemented; harness validated on #2's small initial fixture set, then applied to parser
  outputs.
- **W3 (steps 3–4):** #2 collects upstream semantic/high-level fixtures for #4/#5 while #3
  implementation proceeds (fixture collection is read-only evidence work).
- **W4 (steps 3–7):** CLI skeleton lands in #3, grows in #7 — the skeleton never blocks
  semantic work.
- **W5 (steps 5–6):** #7 QA tooling (matrix-check automation, corpus-runner CI integration,
  acceptance-checklist harness) is designed/built while #5/#6 are implemented.
- **W6 (step 5):** #6 semantic-oracle design notes drafted during #5 implementation (what
  backend-neutral behavior the oracle must validate).
- **W7 (steps 3–7):** support-matrix continuous refinement — each issue PR updates matrix
  states; the final sweep happens in #7.

---

## 4. Branch / PR Plan

Branch names (exact):

| Issue | Branch | Base |
|-------|--------|------|
| #2 | `feat/2-inventory-and-corpus` | `main` |
| #3 | `feat/3-frontend-bootstrap` | `feat/2-inventory-and-corpus` |
| #4 | `feat/4-semantic-core` | `feat/3-frontend-bootstrap` |
| #5 | `feat/5-advanced-semantics` | `feat/4-semantic-core` |
| #6 | `feat/6-typed-hir` | `feat/5-advanced-semantics` |
| #7 | `feat/7-completeness-and-apis` | `feat/6-typed-hir` |

Rules:

- **No pushes to `main`, ever.** All work lands on the branches above; only a maintainer merges,
  and only after review approval.
- One issue = one branch = one Draft PR. Each branch is created by branching off the previous
  branch immediately before it (stacking order 2→3→4→5→6→7). If a base branch is force-updated,
  rebase the stack in order.
- Draft PRs are opened as soon as the branch has reviewable content (not necessarily complete).
  PR title: `feat(N): <short summary>`.
- Each Draft PR targets `main`. Until its base branch merges, its diff includes predecessor
  commits — PR body must state: "Stacked on `feat/N-…` (#PR-N)". Review focus is the top
  commit(s); after the base merges, rebase and the diff narrows to the issue's own changes.
- Merges happen strictly in stack order (2 before 3, … 6 before 7). Never merge a branch whose
  base is unmerged.
- Commits are focused (one logical change per commit), conventional style, and never bundle
  changes from a different issue.
- No PR gets merge-ready status without its validation gates (§5) and QA acceptance checklist
  (§6) passing, evidenced in the PR body.

PR body template:

```markdown
## Scope
Implements <issue N>: <one-line summary>.

## Linked issue
Closes #N (parent: #1)

## Validation evidence
- [ ] `cargo build --workspace` — PASS (CI run: <link>)
- [ ] `cargo test --workspace` — PASS, <count> tests / <count> fixtures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` — PASS
- [ ] <issue-specific check from §5, e.g. matrix-check / corpus run / CLI smoke> — PASS, <output summary>
- [ ] QA acceptance checklist (§6) — PASS (link to checklist output)
- Known limitations / deferred items: <list>

## Stacking
Stacked on `feat/N-…` (#PR-N). Do not merge before its base branch.
```

---

## 5. Validation Gates per Issue

Common gates (every branch, before merge-ready): `cargo build --workspace`, `cargo test
--workspace`, `cargo clippy --all-targets --all-features -- -D warnings`, CI green on the Draft
PR, `cargo fmt --check` if the repo adopts rustfmt, and a QA-role acceptance check against §6.

| Issue | Issue-specific gates |
|-------|----------------------|
| #2 | `support-matrix.toml` parses and passes a mechanical matrix-check (schema + completeness: every entry has provenance, state, VS Code-vs-del-rs classification). Corpus manifest loads; harness runs on the initial fixture set with recorded results. Pinned upstream ref present in `docs/provenance.md`. License/provenance review of every corpus fixture (no unlicensed copies). |
| #3 | Positive/negative parser fixture suites pass. Recoverable-parser robustness: malformed-input corpus (fuzz or curated cases) produces partial trees + structured diagnostics, zero panics. Project-loading tests: imports, cycles, missing files, deterministic provenance. Grep-level check: no `workshop-rs` in any `Cargo.toml`. CLI skeleton smoke: `parse` subcommand returns exit 0 + diagnostics on sample files. |
| #4 | Semantic positive/negative fixture suites pass per construct class (declarations, references, scopes, conversions, calls, operators, access). Diagnostics tests assert spans + cross-file provenance. Provider-boundary test: Workshop-facing name stays unresolved (or bound via provider) without any catalog data. Matrix-check: relevant entries advanced to `semantic-supported`. |
| #5 | Per-feature suites (class/struct/enum, inheritance/override/abstract, generics, lambdas/closures, patterns, recursion) pass. Interaction suite (arrays × inheritance × generics × overload resolution × value/immutable semantics) passes with corpus-backed cases. Inspectability test: identities/relationships queryable from the semantic model. No Workshop encoding types appear in frontend API surface (review gate). Matrix-check: advanced entries `semantic-supported`. |
| #6 | HIR lowering tests for every advanced construct; provenance preserved syntax→semantic→HIR. HIR invariant/validation tests trigger on invalid states with provenance attached. Oracle tests distinguish correct vs incorrect high-level behavior on corpus cases. Crate dependency check: no `workshop-rs`. Adapter-readiness review: a stub consumer can walk the HIR without touching parser/semantic code. |
| #7 | Matrix-completeness check passes: zero non-Workshop-dependent entries below `frontend-supported`. Doctests/API contract tests pass. CLI smoke suite (parse/check/inspect/query on representative corpus projects) passes with stable exit codes + machine-consumable diagnostics (schema/snapshot test). Representative multi-file projects reach resolved semantic/HIR program or expected diagnostics. Known-limitations doc lists every `lowering-dependent`/unsupported entry with rationale. |

---

## 6. Acceptance Checklists (QA role verifies at the end)

Checklist per issue, derived from its acceptance criteria. QA marks each item PASS/FAIL with
evidence (test name, artifact, or PR section).

**#2**
- [ ] Support matrix is machine-readable and mechanically checkable (script passes).
- [ ] Every tracked feature has evidence/provenance + explicit compatibility state (no state
      defaulted without evidence).
- [ ] Corpus contains upstream parser/semantic/high-level tests + representative real-world
      cases, provenance-permitting.
- [ ] VS Code/editor-only behavior is separated from `del-rs` requirements in the matrix.
- [ ] Workshop-dependent cases identified as `lowering-dependent` and never block frontend
      items.
- [ ] Differential harness demonstrated exposing a semantic gap without output-text identity
      (recorded example).

**#3**
- [ ] Standalone Rust project builds and tests (no `workshop-rs`).
- [ ] Frontend-supported syntax parses into documented CST/AST with stable source spans.
- [ ] Invalid/incomplete source yields structured diagnostics + useful partial trees.
- [ ] Multi-file/imported projects load with deterministic provenance + cycle/error handling.
- [ ] Comments/trivia and authored identifiers retained for diagnostics/tooling.
- [ ] Parser fixtures tied to #2 matrix entries (positive + negative).

**#4**
- [ ] Parsed projects resolve into a documented semantic program (no emission).
- [ ] Pos/neg fixtures exist for declarations, references, scopes, conversions, calls,
      operators, access rules.
- [ ] Semantic diagnostics preserve spans + cross-file provenance.
- [ ] Workshop-facing names resolve through a documented provider contract, not copied catalog.
- [ ] Matrix entries advanced to `semantic-supported` where no lowering required.

**#5**
- [ ] Class/struct/enum, inheritance/override, generics, lambda, pattern, recursion semantics
      resolve/diagnose compatibly within declared surface.
- [ ] Cross-feature interaction cases corpus-backed (not isolated syntax tests only).
- [ ] High-level identities/relationships inspectable via semantic model.
- [ ] Runtime-encoding-dependent features represented for HIR/backend without Workshop detail
      leakage.
- [ ] Advanced matrix entries advanced to `semantic-supported`.

**#6**
- [ ] Advanced constructs lower to documented typed HIR without `workshop-rs`.
- [ ] HIR captures allocation/deletion, reference lifetime, virtual dispatch, recursion,
      lambda, storage intent — Workshop-encoding-independent.
- [ ] HIR validation catches invalid states with source provenance.
- [ ] Corpus-backed tests distinguish correct/incorrect high-level behavior pre-backend.
- [ ] A stub Workshop adapter can consume the HIR without parser/semantic redesign.

**#7**
- [ ] All non-Workshop-dependent matrix items evidence-backed + tested.
- [ ] Documented library + CLI APIs parse/check/inspect projects with no Workshop backend.
- [ ] Diagnostics structured, source-attributed, machine-consumable, stable.
- [ ] Cross-file symbols/references, types, semantic relationships, HIR provenance queryable.
- [ ] Representative corpus projects reach resolved semantic/HIR program or expected
      diagnostics.
- [ ] Remaining gaps classified `lowering-dependent` or unsupported with rationale.

---

## 7. Risk Register (top 8)

| # | Risk | L | I | Mitigation |
|---|------|---|---|------------|
| 1 | **Scope size vs budget** — #2–#7 is a full language frontend (lexer→HIR) in one pipeline; the long tail of fixtures/interactions can blow the budget. | High | High | Strict dependency-ordered pipeline with a working end-to-end slice at every step; matrix drives scope (only inventory-backed features are implemented); classify-and-defer beats implement-anyway; each issue PR is a hard checkpoint where remaining budget is re-baselined against #7's completeness bar. |
| 2 | **Corpus licensing** — copying upstream OSTW/DeltinScript tests/examples may violate license or provenance rules. | Med | High | License review in #2/#3 guardrails before any fixture import; every corpus file carries provenance in `docs/provenance.md`; prefer re-expressed/derived test cases over verbatim copies where the license is unclear; if a fixture cannot be imported, record it in the matrix as evidence-pending rather than silently dropping it. |
| 3 | **OSTW vs DeltinScript dialect ambiguity** — the repo covers both `.del` and `.ostw`; upstream behavior may differ by dialect/version, making "compatible" ambiguous. | High | Med-High | #2 pins exactly one reference commit as the initial oracle; dialect divergences are recorded as inventory items with per-dialect evidence; quirks recorded separately from intended semantics (architecture.md); matrix states which dialect a feature's evidence came from. |
| 4 | **Recoverable-parser complexity** — error recovery with useful partial trees + structured diagnostics across the full declared surface is subtle and can consume disproportionate time. | Med | Med | Recovery contract limited to "structured diagnostics + useful partial CST" (issue language), not perfect reconstruction; curated malformed-input corpus + fuzzing as the gate (zero panics); recovery lives in one parser module so it can be tightened without touching semantics. |
| 5 | **HIR backend-neutrality drift** — HIR either leaks Workshop encodings (violating #6 non-goals) or becomes so abstract it is not actionable for #8. | Med | High | HIR design documented in `docs/architecture.md` before implementation; no Workshop types allowed in HIR crate (dependency check); invariants/validation + oracle tests pin observable intent without encoding; #6 acceptance requires a stub adapter consuming HIR — the integration boundary stays the only Workshop-facing seam. |
| 6 | **Overload-resolution complexity** — DeltinScript overload resolution with optional/default args, conversions, and generics interactions can become a research project. | Med-High | Med | Implement only inventory/upstream-evidenced behavior; differential tests against the pinned reference compiler where executable; deterministic documented tie-break rules; exotic cases deferred with explicit matrix state instead of speculative generalization. |
| 7 | **Oracle scope creep** — the #6 "semantic oracle" grows into a second Workshop runtime. | Med | Med | Oracle is explicitly bounded ("minimal", "where practical"); scope is a written feature list approved in #6's PR; oracle reuses the semantic model and HIR; anything requiring Workshop runtime semantics is out of scope and flagged `lowering-dependent`. |
| 8 | **PR review bottleneck** — six stacked Draft PRs create long review queues; review latency stalls the pipeline or encourages shallow review. | Med | Med | One issue per PR with validation evidence in the body (fast review); QA checklist runs before review so reviewers verify claims, not re-run everything; #2/#3 reviewable early; reviews happen in stack order; evidence links (CI runs, fixture counts) make each PR self-contained. |

---

## 8. Definition of Done (whole goal)

1. Issues #2–#7 are implemented, each delivered as its stacked branch + Draft PR targeting
   `main`, PR body linking the issue (`Closes #N`) and carrying validation evidence; every PR
   passed its §5 gates and the §6 QA acceptance checklist; merged in stack order after
   independent review (maintainer performs merges; no pushes to `main`).
2. The compatibility surface is evidence-backed and mechanically checkable: `support-matrix.toml`
   validates; every non-Workshop-dependent entry is at `frontend-supported` or
   `semantic-supported` with tests and provenance; all remaining gaps are explicitly classified
   as `lowering-dependent` or unsupported with rationale in the known-limitations docs.
3. The declared frontend surface parses, resolves, diagnoses, and lowers to typed HIR with
   provenance — all demonstrably without `workshop-rs`; Workshop-facing names bind only through
   the documented provider contract; the integration boundary for #8 is documented and
   unchanged.
4. Public library APIs + CLI let consumers parse/check/inspect projects and query symbols,
   types, semantic relationships, and HIR provenance; diagnostics are structured and stable for
   machine consumers; representative multi-file corpus projects reach resolved semantic/HIR
   programs or expected diagnostics.
5. #8 remains explicitly untouched and blocked (awaiting #7 readiness + `wrightkit/workshop-rs#2`
   public contracts); #9 remains untouched and blocked on #8. No Workshop emission, catalog
   ownership, or end-to-end compatibility claims are made anywhere in #2–#7 deliverables.
6. Repository state: standalone build/tests green on `main` after each merge; `docs/` contains
   architecture.md, syntax-notes.md, inventory.md, provenance.md, support-matrix.toml, and this
   roadmap.
