# CLAUDE.md — contextra-store

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
LSM-Tree-Speicher-Engine: WAL, SST, Manifest, KV-Familien. Größtes Crate des Workspace. Spec: `G-STORE`, `CMP-4`, Kapitel 5 (Datenmodell, Segmente).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1). Ziel laut Spec: ~14.000 Zeilen.
- Umfang: **30.495 Zeilen `src`** (Ist, größter Brocken), 44 Testdateien, 93 `#[test]`.
- Features: `default`, `workspace`, `test-hooks`, `inspect`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`, `docid-128`, `legacy-wal-key`.
- Build: ok.
- Abhängigkeiten (Ist): crypto, durable-fs, env, erasure, ids, mvcc, sys.
- Genutzt von: `contextra-db`, `contextra-bench`.
- `unsafe`: laut `tests/unsafe_islands.rs` in Übergangsphase (Win32-ACL, wandert nach `contextra-sys`).

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-store
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-store --no-default-features   # G-01
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-store -- -D warnings
```

## Ist-Abweichungen zur Spec
- **R2 verletzt (drei E1→E1-Kanten):** `crypto`, `erasure`, `mvcc`. Laut Spec-Ziel hat `store` keine Krypto-Abhängigkeit.
- **G-12 verletzt:** 7 `Instant`/`SystemTime`-Aufrufe außerhalb `contextra-env`.
- **R6 verletzt (grob):** 1.247 `unwrap`/`expect`/`panic`-Treffer inkl. Tests. Produktcode separat zählen, bevor man Zahlen nennt.
- Ziel-Umfang 14.000 Zeilen; Ist 30.495. Aufteilung in Teilcrates ist nicht entschieden.
- Feature-Flags wie `legacy-wal-key`, `fault-injection`, `test-hooks` vor Änderungen gegen SPEC prüfen.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
