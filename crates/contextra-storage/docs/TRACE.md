# R9.1 trace register

This is an evidence register, not a normative source.  The normative source is
`SPEC.md` R9.1. Status means *implementation plus the specified verification*,
never merely that a module compiles.

## Environment and baseline

* 2026-11-23 execution in this workspace supersedes the historical partial
  baseline below: `rustc --version` is `1.85.1`; `rustup` is not installed
  (`rustup toolchain install 1.89.0` returned command-not-found).
* The offline workspace test first failed at dependency resolution (`flatbuffers`
  missing from the local index); after approved Cargo network access, the
  seven-crate baseline check passed. Exact check command:
  `cargo check --ignore-rust-version -p contextra-storage-ports -p contextra-storage -p contextra-crypto-ref -p contextra-keys-file -p contextra-keys-cohort -p contextra-storage-fs -p contextra-storage-bridge --features testing`.
* Exact test command (using writable target storage) passed all unit,
  integration, and doc tests:
  `CARGO_TARGET_DIR=/tmp/contextra-storage-target cargo test --ignore-rust-version --offline -p contextra-storage-ports -p contextra-storage -p contextra-crypto-ref -p contextra-keys-file -p contextra-keys-cohort -p contextra-storage-fs -p contextra-storage-bridge --features testing`.
  The default workspace `target/` is read-only here.
* `cargo clippy`, `cargo fmt`, and `cargo deny` are unavailable as Cargo
  subcommands in the installed toolchain. Their requested baseline gates are
  **not verified**. No gate is declared green from this run.
* `scripts/check_invariant_refs.sh` does run and reports 59 referenced IDs.
  Inspection found that `tests/invariant_refs.rs` supplies those references
  from one `assert!(true)` test; this proves only lexical coverage, not the
  behavioural tests required by the Auftrag. The behavioural mapping below
  remains open pending owner-test assertions.
* `legacy-store-final` was not present in the visible tag list. Creating it
  was attempted at current `HEAD`, but Git could not create the tag lock
  because `.git` metadata is read-only/outside the writable roots.

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
* All seven target crates now expose a deliberately empty `testing` feature,
  so the exact required `cargo {check,test} -p <crate> --features testing`
  baseline is valid.  The full check/test sweep passed after this change.
* Initial baseline also found the workspace target directory read-only under
  this sandbox; the explicit temporary target directory above is required.

## Work-package matrix

### Aktuelle SPEC-basierte Änderungen (noch unverifiziert)

* P0 / WP-01.2 und WP-03.1/03.2 sind in Arbeit: `Clock` liefert jetzt eine
  monotone Zeit, UTC, dauerhaften UTC-Floor und Skew-Status; der Floor wird per
  monotonem CAS durabel fortgeschrieben. Deadline-UTC wird bei
  Rollback/Unsicherheit fail-closed abgewiesen. Der KeyCache verwendet
  weiterhin ausschließlich monotone Laufzeit.
* `Fs`-Ports enthalten jetzt geordnetes `read_ranges` in caller-owned Puffern,
  RAII-Locks und optionale Hardlink/Reflink/Hole-Punch-Fähigkeiten. `MemFs`
  implementiert den prozesslokalen Lock; `FaultFs` zählt und injiziert Fehler
  bei diesen Dateisystemoperationen. Die Default-Range-Lesung läuft über
  `read_at`, sodass jede Teiloperation im Fault-Sweep bleibt.
* Fehler `Error::Io(Interrupted)` wird transient klassifiziert; andere rohe
  I/O-Fehler werden nicht automatisch als transient wiederholbar eingestuft.
  `Erased{at}` wurde als typisierter Aufruferfehler ergänzt.
* WP-04.1/04.2 werden ebenfalls gegen SPEC §5.2/§9.4 ausgerichtet: direkte
  Provider-Operationen (`info`, `kek`, `create_cluster_key`, `cluster_key`,
  `probe`, `list`, `destroy_many`) ersetzen die zuvor nichtnormative
  Batch-/`KeyProbe`-Signatur. Die In-Memory-Implementierung liegt jetzt im
  `contextra-crypto-ref`-Crate; Recovery unterscheidet `Disabled`, bestätigtes
  `Absent` und `KeyLost` ohne Erasure-Beleg. `KeyProviderConformance` deckt
  Epoch-Zerstörung und Providerzustände ab.
* Metriknamen und -dimensionen sind nun geschlossene Port-Enums; die
  Metrikdefinitionen und Sample-/Labelprüfung richten sich nach den in SPEC
  §16 genannten Dimensionen. Standard-Observer-Events nehmen keine Strings
  oder beliebigen Payloads entgegen.
* WP-02.1 ist im SPEC-Abgleich: Tag-Codec lehnt Stamp 0 (SPEC §4.2: erster
  Stamp 1) und reservierte Merge-Tags ab. Neue Primitive-Tests decken
  kanonische Varint-Grenzwerte, CRC32C-Checkvektor und Entry-Ordnung ab;
  Verifikation ist bis zum vereinbarten Schlusslauf offen.
* Neue Port-/Storage-Tests decken diese Verträge ab. Schlusslauf nach den
  Änderungen: der 7-Crate-`cargo check`-Befehl und der entsprechende
  `cargo test`-Befehl (siehe oben unter „Environment and baseline“) sind
  erfolgreich. Die Gate-Beweise bleiben dennoch unvollständig.
* Der bisherige Governor-Stub ist durch eine atomare Admission-Komponente mit
  DiskFull-Eintritt und Recovery-Hysterese ersetzt. Er sperrt normale Writes
  und Wartung im Notbetrieb, lässt Reads und `UrgentErase` zu und validiert
  den konfigurierten Maintenance-I/O-Anteil. `tests/governor_hysteresis.rs`
  zitiert `INV-DISK-1`; Kompilierung, Fault-Sweep-Integration und Scheduler-
  Anbindung bleiben offen, daher ist WP-03.7 weiterhin `teilweise`.
* Der aktuelle Schlusslauf 2026-11-23 bestand für alle sieben Storage-Crates
  mit `--ignore-rust-version --offline --features testing`; der vollständige
  Integrationstest- und Doc-Test-Lauf war grün. Die Tests decken die zuletzt
  ergänzten Clock-, Error-, Fs-, KeyProvider-, Metrik-, Codec- und
  Governor-Verträge ab. Das belegt nur die vorhandene Testsuite, nicht die
  fehlenden Modell-, Crash-Matrix-, Fuzz-, Benchmark- und Review-Anforderungen;
  kein Gate P0–P8 wird hierdurch als bestanden erklärt.

| WP | Owner | SPEC | INV | Modules | Current state / evidence | Tests / gate |
|---|---|---|---|---|---|---|
| WP-01.1 | 01 | §3.2, §4, §12 | ID, LIM, SEAL | ports/{ids,change,error} | partly: `DataClass`, instance/durable position, erase/tomb IDs are now port-owned and redacted; remaining value-contract audit open | `port_values`, redaction; G-0 open |
| WP-01.2 | 01 | §3.2 | LOCK, KEY | ports/traits | partly: `AnchorStrength` and optional `Authority` port added with mock compile test; remaining Fs/extension contract audit open | `port_values`, layering; G-0 open |
| WP-01.3 | 01 | §3.5 | all applicable | ports/facade | missing: reviewer/API freeze | G-0 EXTERN-OFFEN |
| WP-02.1 | 02 | §5.1–5.2 | FMT | codec/{varint,tags,checksums,comparator} | partly: codecs/tests exist; full fuzz/goldens absent | storage codec tests; G-0 open |
| WP-02.2 | 02 | §5.2 | SEAL | codec/seal | partly: direct-key seal test exists; complete raw-copy matrix absent | seal_frame; P1 open |
| WP-02.3 | 02 | §5.3 | FMT | codec/header,keys_codec | partly | golden vectors absent; G-F open |
| WP-03.1–03.8 | 03 | §3, §11 | LOCK, DISK | env/,sync/ | partly: MemFs/FaultFs/ranked exist; RealFs/Sim/governor/platform evidence incomplete | faultfs/lock tests; P0–P8 open |
| WP-04.1–04.6 | 04 | §5.2, §8 | KEY | crypto-ref,keys*,keys/cache | partly: reference cipher/cache exist; HMAC now uses audited `hmac` crate rather than a local construction; conformance/adapters absent | HMAC KAT, seal/key-cache tests; P0/P6 open |
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
| WP-15.1–15.9 | 15 | §14–17 | all | testing/,tools/reference,formal | partly: invariant checker now expands SPEC §15 and passes; harness/oracle/reference/gates incomplete | G-0–G-P open |
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

The checker now verifies that every SPEC §15 ID is cited in a compiled test
source.  Citation is traceability only: behavioural proof remains open unless
the row names an assertion-bearing owner test.

| Invariant family | Current cited test |
|---|---|
| All 59 IDs | `tests/invariant_refs.rs::spec_r8_1_invariant_references_are_compiled_with_the_test_suite` (citation baseline; not behavioural proof) |
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
* `OD-02`: SPEC §3.2 requires an explicit anchor strength while the prior port
  had none. Choice: `Anchor::strength()` is mandatory, and `MemAnchor` reports
  `Weak`; no local anchor is treated as rollback-proof. Covered by
  `port_values.rs`.
* `OD-03`: The existing HMAC-SHA256 implementation duplicated the construction
  locally, contrary to NZ-2. Choice: use `hmac` 0.12 directly and make
  `Mac::tag` fallible so a provider/setup failure cannot become a fabricated
  tag. The direct dependency is locked in `Cargo.lock`; the fixed-key HMAC KAT
  remains the regression test.

