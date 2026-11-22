# CLAUDE.md — contextra-durable-fs

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Crash-sichere Dateisystemoperationen und synchrones Flushen auf Platte. Grundlage für Durability (INV-FSYNC-1). Der frühere Tracer `tools/fsync_durability_tracer.py` wurde entfernt (Historie: `git show 0ec4de767:tools/fsync_durability_tracer.py`).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E0** (SPEC §3.1). Keine internen Abhängigkeiten.
- Umfang: 1.910 Zeilen `src`, 13 Testdateien, 43 `#[test]`.
- Features: `default`, `std`, `crc`, `serde`, `tracing`, `inspect`.
- Build: ok.
- Genutzt von: `contextra-erasure`, `contextra-store`, `contextra-vector`, `contextra-db`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-durable-fs
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-durable-fs -- -D warnings
```

## Ist-Abweichungen zur Spec
- REM-13: **Fault-Injection und `Poisoned` sollen hier raus** und nach `contextra-testkit` (FaultVfs) wandern. Aktuell noch enthalten.
- Feature `inspect` prüfen, ob es nach REM-13 noch nötig ist.
- Default-Feature `std` ist in R3 begründet (Ausnahme), bleibt also.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
