# deltin-rs architecture — historical compatibility pointer

The current architecture contracts are maintained under
[`docs/architecture/`](architecture/README.md):

- [`language-core.md`](architecture/language-core.md) — DEL/OSTW core scope, semantic ownership, typed implementation model, and feature locality;
- [`workshop-boundary.md`](architecture/workshop-boundary.md) — DEL runtime/lowering ownership and the canonical Workshop boundary.

This path is retained for existing links. The previous ~95 KB "implemented baseline" mixed point-in-time crate layout, dependency versions, support-matrix state, API sketches, implementation plans, and design decisions. It is preserved in Git history but is no longer a current architecture authority.

Current support state comes from `support-matrix.toml`, compatibility/corpus evidence, and real-project verification. Current implementation reality comes from source, Cargo metadata, tests, and integrations. Neither is established by the historical architecture document alone.
