# R8.1 trace register

This is an evidence register, not a normative source.  The normative source is
`SPEC.md` R8.1.  Status means *implementation plus the specified verification*,
never merely that a module compiles.

## Environment and baseline

* 2026-11-23: installed compiler is `rustc 1.85.1`; `rustup` is unavailable.
  All locally reproducible commands therefore use `--ignore-rust-version
  --offline` and `CARGO_TARGET_DIR=/tmp/contextra-storage-target`.  This is
  `EXTERN-OFFEN` against the required 1.89.0 pin.
* The initially offline registry lacked `chacha20poly1305` and
  `crossbeam-skiplist`; `cargo fetch --locked` restored the declared dependency
  cache.  No dependency was added or changed.
* Storage baseline: `cargo test --ignore-rust-version --offline -p
  contextra-storage --features testing -- --nocapture` passed: 60 integration
  tests, 0 failed (2026-11-23).  It is only a partial baseline, not a gate.
* The non-storage crates do not define a `testing` feature.  Their required
  baseline commands are recorded as `cargo {check,test} -p <crate>` without
  that feature; the original feature invocation is invalid Cargo syntax for
  those package manifests.
* Initial baseline also found the workspace target directory read-only under
  this sandbox; the explicit temporary target directory above is required.

## Work-package matrix

| WP | Owner | SPEC | INV | Modules | Current state / evidence | Tests / gate |
|---|---|---|---|---|---|---|
| WP-01.1 | 01 | §3.2, §4, §12 | ID, LIM, SEAL | ports/{ids,change,error} | partly: types exist; complete contract audit absent | redaction; G-0 open |
| WP-01.2 | 01 | §3.2 | LOCK, KEY | ports/traits | partly: traits exist; mock-conformance absent | layering; G-0 open |
| WP-01.3 | 01 | §3.5 | all applicable | ports/facade | missing: reviewer/API freeze | G-0 EXTERN-OFFEN |
| WP-02.1 | 02 | §5.1–5.2 | FMT | codec/{varint,tags,checksums,comparator} | partly: codecs/tests exist; full fuzz/goldens absent | storage codec tests; G-0 open |
| WP-02.2 | 02 | §5.2 | SEAL | codec/seal | partly: direct-key seal test exists; complete raw-copy matrix absent | seal_frame; P1 open |
| WP-02.3 | 02 | §5.3 | FMT | codec/header,keys_codec | partly | golden vectors absent; G-F open |
| WP-03.1–03.8 | 03 | §3, §11 | LOCK, DISK | env/,sync/ | partly: MemFs/FaultFs/ranked exist; RealFs/Sim/governor/platform evidence incomplete | faultfs/lock tests; P0–P8 open |
| WP-04.1–04.6 | 04 | §5.2, §8 | KEY | crypto-ref,keys*,keys/cache | partly: reference cipher/cache exist; conformance/adapters absent | seal/key-cache tests; P0/P6 open |
| WP-05.1–05.4 | 05 | §5.4, §9 | LOG | store/redo | partly: frame/reader/writer and sweep exist; full control/gate bodies absent | crash_sweep_redo; P1/P2 open |
| WP-06.1–06.5 | 06 | §5.5, §9 | CAT, ID, STAMP | catalog/ | partly: edits/replay/reservation exist; complete snapshot/loom missing | catalog_crash_sweep; P1–P3 open |
| WP-07.1–07.5 | 07 | §5.6 | PL, SEAL | store/run | partly: blocks/reader/builder/cache exist; entry builder/full reader absent | run_* tests; G-F open |
| WP-08.1–08.4 | 08 | §5.7 | BLOB | store/blob | partly: record and GC index exist; writer/reader completion absent | blob_*; P4/P5 open |
| WP-09.1–09.4 | 09 | §6 | VIEW, GRAVE | engine/{memtable,version,registry,graveyard} | partly | mvcc/graveyard; P1–P5 open |
| WP-10.1–10.6 | 10 | §6, §11 | ATOM, COMMIT, GUARD, VOL | engine/committer* | partly | mvcc_commit_path; P1–P3 open |
| WP-11.1–11.5 | 11 | §7 | VIEW, VIS | engine/{views,cursor,page,cache} | partly | no model equivalence; P1–P4 open |
| WP-12.1–12.4 | 12 | §8, §12 | CLUSTER, ERA-10/12 | erase/cluster_table,links,restrictions | partly | cluster_table; P1/P6 open |
| WP-13.1–13.6 | 13 | §9 | ERA-1…12 | erase/{gate,protocol,tickets,receipt} | partly: no end-to-end protocol proof | P1/P6 open |
| WP-14.1–14.3 | 14 | §10 | ANCH, LOG | recovery/ | partly | recovery matrix absent; P1/P6 open |
| WP-15.1–15.9 | 15 | §14–17 | all | testing/,tools/reference,formal | partly: harness/oracle skeleton; invariant checker/reference/gates incomplete | G-0–G-P open |
| WP-16.1–16.5 | 16 | §6, §17 | CLAIM, PL | maint/ | partly | no maintenance model evidence; P2/P5 open |
| WP-17.1–17.5 | 17 | §12.4 | CONS, VIS | engine/{segments,extension,feed} | missing/partial | P5/P6 open |
| WP-18.1–18.5 | 18 | §13 | — | storage-bridge | missing/partial | P6/P8 open |
| WP-19.1–19.3 | 19 | §9.4 | ERA-2/3/6/11 | maint/purge | partly | catalog purge tests only; P2/P6 open |
| WP-20.1–20.3 | 20 | §9.5 | LED, ANCH | erase/ledger | missing/partial | P1/P6 open |
| WP-21.1–21.4 | 21 | §12.6 | BR-1…4 | engine/branches | missing/partial | P7 open |
| WP-22.1–22.3 | 22 | §10.5 | ERA-9, ANCH | checkpoint/ | missing/partial | P6 open |
| WP-23.1–23.4 | 23 | §3.5, §16 | LIM, DISK | lib.rs,config,spaces,metrics | partly: open/config validation exists | config_validate; P1/P8 open |
| WP-24.1–24.7 | 24 | §13, §18 | ERA, PRIV | tools/migrate | missing: only README | G-CUTOVER EXTERN-OFFEN |
| WP-25.1-gap | 25 | §12.8 | all audit-owned | SPEC audit remediation | missing: no per-E audit evidence matrix | pre-G-0/G-F open |

## Invariant mapping

Every listed invariant remains unproven until this row names a test with an
`INV-*` citation.  The current test names frequently lack that citation, which
is itself a WP-15 defect.

| Invariant family | Current cited test |
|---|---|
| INV-STAMP-1/2/3, INV-ID-1/2, INV-ATOM-1, INV-COMMIT-2, INV-GUARD-1 | missing |
| INV-VIEW-1/2, INV-VIS-1, INV-CONS-1 | missing |
| INV-LOCK-1/2 | `tests/lock_order.rs` (citation must be added) |
| INV-CAT-1/2 | `tests/catalog_crash_sweep.rs` (citation must be added) |
| INV-FMT-1 | codec tests (citation missing) |
| INV-LOG-5/7/8/9 | `tests/crash_sweep_redo.rs` (citation missing) |
| INV-GRAVE-1/2 | `tests/graveyard.rs` (citation missing) |
| INV-CLAIM-1, INV-BLOB-1, INV-PL-1…5, INV-CLUSTER-1/2 | missing |
| INV-SEAL-1 | `tests/seal_frame.rs` (citation missing) |
| INV-ERA-1…12 | missing |
| INV-LED-1, INV-ANCH-1 | missing |
| INV-KEY-1…4 | `tests/key_cache.rs` (citation missing) |
| INV-PRIV-1, INV-VOL-1, INV-LIM-1, INV-DISK-1 | missing |
| INV-BR-1…4 | missing |

## Performance mapping

| H-n | Benchmark / measurement |
|---|---|
| H-1–H-22 (except absent H-14) | missing: no reproducible benchmark report yet |

## Open decisions and external blockers

* `EXTERN-OFFEN-01`: Rust 1.89.0 toolchain cannot be installed because `rustup`
  is absent.  Temporary choice: use 1.85.1 with `--ignore-rust-version`; no
  gate is declared from that evidence.
* `EXTERN-OFFEN-02`: G-0 requires two human reviewers; G-F requires freeze
  review; P8 requires crypto/threat-model review.  These cannot be self-approved.
* `OD-01`: manifests expose `testing` only on `contextra-storage`, contrary to
  the requested blanket command.  Choice: test that crate with the feature and
  test the remaining target crates without it; add feature parity only if it is
  required by the normative interface.

