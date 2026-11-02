# Bauen, Prüfen und Gates

Maßgeblich: [`SPEC.md`](SPEC.md) Abschnitte 5.1, 5.8, 14.7, 14.8, 15.1 bis 15.3, D-82, D-93.

## 1. Toolchain

- `rust-toolchain.toml` pinnt derzeit **1.89.0** mit den Komponenten `rustfmt` und `clippy`.
- **Sollwert (D-82):** Build-Toolchain ist die zum Release-Schnitt aktuelle Stable-Version, höchstens zwei Minor-Versionen dahinter. Der Pin 1.89.0 erfüllt diese Regel nicht. Die Aktualisierung ist Arbeitspaket WP-0.2.
- **MSRV (D-82):** `rust-version` ist die älteste Version der CI-Matrix. Derzeit steht im Workspace-Manifest `1.89`. Die MSRV wird nur in Minor-Releases angehoben.
- Release-Builds laufen mit `--release` und `lto = "thin"`. Benchmarks werden auf der Referenzhardware (SPEC 14.8) mit Hardware-Protokoll ausgeführt.

## 2. Plattformen

- **Tier 1:** Linux x86_64 und aarch64 auf ext4 oder XFS.
- **Tier 2:** macOS (APFS, `F_FULLFSYNC`) sowie Linux auf btrfs oder ZFS. Für diese Dateisysteme gilt das Restrisiko `FsJournalOrCow` immer.
- **Nicht unterstützt:** Windows, NFS, SMB, FUSE, overlayfs. Die Startprobe verweigert den Start (D-56, FM-11).

## 3. Workspace

Der Soll-Workspace (SPEC 5.1) besteht aus 14 Produktionscrates sowie `contextra-testkit`, `contextra-lab`, `ctx-verify` und `contextra-arch-tests`. Im Ist-Zustand fehlen `contextra-erasure` als Workspace-Mitglied, `contextra-lab` und `ctx-verify` (SPEC F-04, F-33). Die Crate-Namen stehen in den jeweiligen `Cargo.toml`.

## 4. Prüfbefehle

Diese Befehle sind die Prüfungen des Gates A (SPEC 15.3) und des Tests T-BUILD-1:

```bash
# Formatierung
cargo fmt --all -- --check

# Build aller Targets mit eingechecktem Lockfile
cargo check --workspace --all-targets --locked

# Lints (Gate G-C erweitert dies um die Zero-Panic-Lints)
cargo clippy --workspace --all-targets --locked -- -D warnings

# Tests
cargo test --workspace --locked

# Architektur- und Lint-Tests
cargo test -p contextra-arch-tests --locked
```

Ab Gate G-C gelten zusätzlich `-D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::indexing_slicing` für die Kern-Crates. Der Ratchet erlaubt nur sinkende Zahlen (SPEC 5.8).

**Lockfile.** `cargo check --workspace` ohne `--locked` aktualisiert `Cargo.lock`. Die Änderung gehört in denselben Commit wie die Manifest-Änderung. Danach ist die Prüfung mit `--locked` zu wiederholen (D-93).

**Abhängigkeiten und Netzwerkbann.**

```bash
cargo deny check
cargo tree -e normal   # darf keine Netzwerk-Bibliotheken enthalten (HR-1, D-54)
```

Erlaubte Tokio-Features: `rt-multi-thread`, `sync`, `time`, `macros` (D-54). Verboten sind `full`, `net`, `fs`, `process`, `signal`, `io-std`.

**Speichersicherheit (T-UNSAFE-1, SPEC 15.1).** Miri läuft auf `contextra-types`, `contextra-mvcc` und der Skalar-Referenz von `contextra-simd`. AddressSanitizer läuft auf `contextra-sys`.

```bash
cargo +nightly miri test -p contextra-types -p contextra-mvcc
cargo +nightly miri test -p contextra-simd
RUSTFLAGS=-Zsanitizer=address cargo +nightly test -p contextra-sys --target x86_64-unknown-linux-gnu
```

**Weitere Prüfungen (ab den jeweiligen Arbeitspaketen).** `cargo-mutants` auf Erasure, Klassifikation, Matcher, Gate und Tresor (T-MUT-1; überlebende Mutanten blockieren G-D). `cargo-llvm-cov` mit Zeilenabdeckung ≥ 90 % in `erasure`, `crypto`, `store` und `ports` und ≥ 95 % in der Entscheidungslogik von `erasure` (SPEC 15.1). `cargo-semver-checks` (D-66). `cargo-fuzz` für alle Parser aus Anhang F. `loom` für Commit-Barriere, Gate und Purge-Barriere (T-CONC-1).

**Benchmarks.** Der Harness `ctx-bench` (WP-6.1) ist noch nicht vorhanden. Ein Lauf ohne vollständiges Hardware-Protokoll ist ungültig (P9, SPEC 15.2).

## 5. Aktueller Zustand

Der Workspace baut im Ist-Zustand nicht vollständig (SPEC F-32, Prüfung mit `cargo check`):

- `contextra-store`: vier Fehler. Der Port `license` wird in `src/lsm/config.rs` referenziert, existiert aber nicht mehr. `unsafe fn contextra_sys::mmap_readonly` wird zweimal in `src/wal/replay.rs` ohne `unsafe`-Block aufgerufen.
- `contextra-vector`: ein Fehler. Derselbe `unsafe`-Aufruf in `src/persistence/mmap.rs`.
- `contextra-engine` und `contextra` wurden nicht erreicht. Ältere Befunde (F-01, 33 Fehler) sind nicht bestätigt, aber nicht ausgeschlossen.
- `Cargo.lock` ist veraltet. `cargo check --locked` bricht ab.

Fehlerfrei übersetzen die Lib-Targets von `types`, `ports`, `sys`, `crypto`, `mvcc`, `simd`, `text`, `rank`, `durable-fs` und `arch-tests`.

Die Behebung ist Arbeitspaket WP-0.3 (SPEC 15.4). Mit `mmap` wird nicht gelöst; die Lesezugriffe laufen über `Vfs` (D-94).

## 6. Gates

| Gate | Bedingung (Auszug) |
|---|---|
| G-A | `check --workspace --all-targets --locked` und `fmt --check` grün (D-93) |
| G-B | Rückbau nach D-84 vollständig; Features und Tokio nach D-54; `deny` ohne offene Ausnahmen; ein `Vfs`-Trait |
| G0 | Speicherentscheid (SPEC 11.9) mit Messprotokoll |
| G-C | Kern-Crates ohne `unwrap`, `expect`, `panic!`, `let _ =`, `allow`; Clippy mit `-D warnings` |
| G-D | Löschkern S0: T-ISO, T-GATE, T-VAULT, T-KEYCOV, T-FK, T-CRASH, T-RAW, T-PROOF, T-RESTORE grün; Lab MK1 bis MK3; Mutationstests ohne Überlebende |
| G-E | S1 Textindex |
| G-F | Keyslots, HEADER, Plattformprobe |
| G-G | S2 Vektorsuche |
| G-H | Release S0 bis S3 bzw. Cut-Linie; alle T- und BM-IDs; SBOM; signierte Artefakte |
| G-LEGAL | Extern (SPEC 6.7); blockiert nur Löschaussagen über 6.4 hinaus |

Reihenfolge: A → B → G0 → C → D → F → E → G → H.
