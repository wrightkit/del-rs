# OSTW pinned-reference evidence

This directory is the owner-side, evaluation-only evidence package for
`deltin-rs`. It was migrated from `wrightkit/wright/compatibility/ostw/`; the
corpus file identities, probe observations, reference hashes, and declared
roots are retained so Wright can remove its duplicate owner infrastructure in
the follow-up migration.

The OSTW binary is downloaded to `target/ostw-reference/`; it is neither
committed nor a Cargo dependency. `reference.json` is the canonical machine
identity for the pinned release asset; the recorded results and probe manifests
must refer to that same identity.

The corpus sources are byte-identical to the current Wright copy. The
`protect-ban/main.ostw` manifest hash is corrected to the actual migrated bytes
because Wright's pre-existing `corpus.json` entry was stale; recorded reference
observations are unchanged.

`reference.json` pins the release tag, immutable tag commit, release asset size,
and SHA-256. `latest` is never an evidence identity.

```sh
python3 compatibility/ostw/run_oracle.py --acquire --ping
python3 compatibility/ostw/run_oracle.py --acquire --update
python3 compatibility/ostw/run_oracle.py
python3 compatibility/ostw/run_oracle.py --probes
python3 compatibility/ostw/run_oracle.py --check
```

`--check` validates the committed evidence without downloading or executing the
upstream reference. `--acquire --ping` is the bounded reference smoke check;
`--update` and `--probes` are explicit maintainer operations that re-record
observations only after reviewing drift.

The runner drives `Deltinteger --langserver` using Content-Length framed JSON-RPC.
It opens a project workspace so `ds.toml` is visible, records diagnostics and the
custom `workshopCode` / `elementCount` notifications, and writes deterministic
JSON evidence. It never invokes the clipboard-bound default compiler path.

## Explicit compile/document roots

Pinned P1 evidence (#118) established that the upstream LSP compiles the
**last-opened document plus its transitive import closure**; `ds.toml.entry_point`
is not the LSP compile selector. Every recorded observation is therefore
produced by a session that opens exactly one document — the observation's
explicit `root` — so the result can only be that root's compile and can never
acquire meaning from `didOpen` ordering. `corpus.json` lists the reviewable
`roots` (with `entry-root` / `document-root` / `historical-document-root`
roles) per project; `results.json` (schema v2) records one observation per
root: accept/reject, `elementCount`, full source-located diagnostics, the
import-closure identity, and missing-import boundaries.

`probe.json` manifests may designate a probe as `differential-target`; the
runner aggregates accepted targets under `differentialTargets` in
`probes/results.json` and refuses to list a target the pinned reference
rejects. These are the immutable, reference-accepted forward-compilation
targets for #119.

## Determinism

The langserver debounces compiles ~50 ms after the last `didOpen` and publishes
one coherent `workshopCode`/`elementCount`/`publishDiagnostics` triple per
compile. The runner drains until the server is quiet (`QUIET_SECONDS`, default
3 s) and records only the LAST compile triple — deterministic because the open
set is explicit and fixed per observation. A session that drops mid-stream
(transient container failure) is retried; the recorded triple is unchanged.

The corpus runner re-run twice produces byte-identical `results.json`; running
it without `--update` fails with `OSTW_ORACLE_DRIFT` when the recorded
evidence no longer reproduces under the pinned reference. That drift check is
a manual maintainer command, not a native CI merge gate: the upstream-reference
CI job was removed in Wright #177. Consumer CI may consume these recorded
snapshots, but does not re-derive them. `accept`/`reject` is derived from the
final `elementCount` (`>= 0` means
the reference produced Workshop code; `-1` means it reported errors).

## Ownership and downstream residuals

The authoritative package is exactly `reference.json`, `corpus.json` and
`corpus/`, `results.json`, `probes/`, and `reconstruction/`. Together they own
the pinned reference identity, corpus sources and hashes, recorded observations,
probe outputs, and the reconstruction boundary. No Wright-side copy of this
owner infrastructure is required for deltin-rs reproducibility.

There is no required Wright-side residual in the migration source's current
`origin/main`: its `del-integration` job runs only the `wright-ostw` adapter
tests, and that crate has no test target reading `compatibility/ostw/`. Wright
#182 can therefore remove the full owner-style package. If a later consumer
adds a snapshot, it must name the file and test that reads it; such a snapshot
would remain consumer-owned and would not become a second contract.
