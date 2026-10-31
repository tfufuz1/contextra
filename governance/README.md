# governance/ — Single Source of Truth für Gates & Policies

<!-- doc-stamp: 2026-10-31 | Normativ: docs/SPEC.md §17, §18.2 | PLAN.md §12 -->

> [!IMPORTANT]
> Dieses Verzeichnis ist ein **geschützter Pfad** (`governance/protected-paths.toml`).
> Änderungen an diesen Dateien erfordern ein formales ADR im Commit und das PR-Label `protected-path`.

---

## 1. Zweck & Übersicht

Das `governance/`-Verzeichnis definiert maschinell durchsetzbare Qualitäts-, Sicherheits- und Architekturregeln über **native Cargo-Tests, Clippy-Lints und Justfile-Targets** (`just check`, `just test`, `just gates`).

## 2. Dateien & Verantwortlichkeiten

| Datei | Zweck | Normativität |
|---|---|---|
| [`gates.toml`](gates.toml) | Vollständiger Katalog aller Quality Gates (Kompilierung, Lints, Layering, Coverage). | SPEC §17 |
| [`ratchet.toml`](ratchet.toml) | Qualitäts-Baselines (Grenzwerte für Warnungen, Zeilenzahlen, Unsafe-Zähler). Dürfen nur sinken. | SPEC §17, P7 |
| [`protected-paths.toml`](protected-paths.toml) | Liste geschützter Pfade, die besondere Review-Rechte erfordern. | REQ-GOV-002 |
| [`security_ports.toml`](security_ports.toml) | Verzeichnis aller sicherheitskritischen Ports (`KmsProvider`, `Vfs`, `ErasureJournal`). | INV-WIRING-2 |
| [`key_purposes.toml`](key_purposes.toml) | Feste Zweckbindung und Labels für kryptografische KDF-Ableitungen. | INV-KEYPURPOSE-1, SPEC §12.3 |
| [`clippy.toml`](clippy.toml) | Workspaceweite Clippy-Konfiguration: Verbotene Methoden (`Instant::now`, `thread_rng`) und Typen. | SPEC §17.3, P28 |
| [`determinism-allow.toml`](determinism-allow.toml) | Explizit dokumentierte und genehmigte Ausnahmen für Nicht-Determinismus (z. B. System-RNG für Krypto-Keys). | P28, ADR-098 |

---

## 3. Rangfolge der Governance (docs/SOURCE_OF_TRUTH.md)

1. **`docs/SPEC.md`** — Normativ, oberste Quelle der Wahrheit.
2. **`docs/VETOES.md`** — Veto-Register und architektonische Verbote.
3. **`capabilities.toml`** — Crate-Ring- und Klassenzuordnung.
4. **`governance/gates.toml`** — Gate-Definitionen.
5. **`PLAN.md`** — Umsetzungsstrategie und Härtungsfahrplan.
6. **`governance/ratchet.toml`** — Baselines.

---

## 4. Ausführung der Gates im Dev-Loop

```bash
# Prüfung der Architekturschichtung (Ringmodell 0-4 DAG)
cargo test --test layering --locked

# Prüfung auf Unsafe-Inseln
cargo test --test unsafe_islands --locked

# Prüfung auf Einhaltung der Clippy-Verbote (Zero Panic, P28 Determinismus)
cargo clippy --workspace --all-targets -- -D warnings
```
