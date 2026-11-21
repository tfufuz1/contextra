# CLAUDE.md — contextra-ports

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Kanonische, dyn-kompatible **Port-Traits** und DTOs für alle Subsysteme. Ein Port-Trait ist genau hier definiert (R9); implementiert wird es in dem Crate, das den Zustand besitzt (SPEC §3.4).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E0** (SPEC §3.1, ADR-32, ADR-33).
- Umfang: 6.755 Zeilen `src`, 3 Testdateien, 30 `#[test]`.
- Features: `default`, `conformance`, `contract-tests`, `testkit`.
- Build: ok.
- Genutzt von: checkpoint, contextra, db, graph, mcp, mvcc, rank, testkit, text, vector.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-ports
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-ports --features conformance
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-ports -- -D warnings
```

## Ist-Abweichungen zur Spec
- **R1 verletzt:** Laut Spec-Ziel hängt `ports` nur von `ids` ab. Aktuell zusätzlich `contextra-env` und `contextra-types` (Cargo.toml).
- Ports sind **Breaking-Change-Quelle** für ~10 Crates. Jede Signatur-Änderung in der Notiz der betroffenen Dependents als „Offen“ vermerken.
- 74 `unwrap`/`expect`/`panic`-Treffer (grob, inkl. Tests), R6 beachten.
- Ziel-Umfang laut Spec: ~3.000 Zeilen (Ist 6.755).

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
