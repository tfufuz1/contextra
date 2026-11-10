# Handoff FND-1 — Fundament: Mikro-Kerne contextra-ids und contextra-env

**Datum:** 10. November 2026
**Commit-Kennung:** FND
**Branch:** `jules/fnd-1-ids-env`
**Status:** Erfolgreich abgeschlossen

---

## 1. Ergebnis je Arbeitspaket (WP)

| Arbeitspaket | Status | Prüfbefehl | Ergebnis |
|---|---|---|---|
| **FND.0: Baseline & Inventar** | GRÜN | `cargo metadata`, `cargo tree`, `cargo check` | Baseline-Dateien unter `docs/decoupling/baseline/workspace/` konsolidiert erfasst. 88 `::from_key(`-Aufrufer außerhalb von `/tests/`, 9 `ContextraError::InvalidInput`-Stellen in IDs/Refs inventarisiert. |
| **FND.1: Golden-Format-Korpus** | GRÜN | `cargo test -p contextra-types --test golden_ids_parity --locked`<br>`cargo test -p contextra-ports --test golden_env_parity --locked` | Golden-Dateien für `from_key` (64/128-Bit), serde-Ausgaben (`DocId`, `EntityId`, `TxId`, `TenantId`, Refs in JSON/Bincode), alle 43 `ContextraError`-Varianten und `SeededRng`-Folgen unter `crates/contextra-ids/tests/golden/` und `crates/contextra-env/tests/golden/` mit `MANIFEST.sha256` bitgleich verifiziert. |
| **FND.2: contextra-ids** | GRÜN | `cargo test -p contextra-ids --locked`<br>`cargo test -p contextra-types --locked`<br>`cargo test -p contextra-ids --features docid-128 --locked` | `contextra-ids` als autarker Mikro-Kern angelegt (`#![forbid(unsafe_code)]`, nur `blake3`, `serde`, `thiserror`). Differenztests über 10.000 Schlüssel gegen wörtlich eingefrorene Referenzkopien aus `types` und `vector` bitgleich grün. `contextra-types` re-exportiert alle Typen transparent. |
| **FND.3: contextra-env** | GRÜN | `cargo test -p contextra-env --locked`<br>`cargo test -p contextra-ports --locked` | `contextra-env` angelegt (nur `std`, `#![forbid(unsafe_code)]`, Ports `Clock`, `Rng`, `IdGen`, `MetricsSink`, Implementierungen `SystemClock`, `unix_ms`, `SeededRng`, `SequentialIdGen`, `NoopMetricsSink`, `InMemoryMetricsSink`, `ManualClock` unter `test-utils`). Differenztest `unix_ms` gegen Referenzkopien aus `text`, `graph`, `vector` bestanden. `contextra-ports` re-exportiert alle Symbole transparent. |
| **FND.4: Schlussprüfung** | GRÜN | `cargo tree -p contextra-ids -e normal --no-default-features`<br>`cargo tree -p contextra-env -e normal --no-default-features`<br>`cargo clippy -p contextra-ids -p contextra-env -p contextra-types -p contextra-ports --all-targets -- -D warnings` | Null-Prüfungen: 0 interne Workspace-Abhängigkeiten in `contextra-ids` und `contextra-env`. Keine unerlaubten Erwähnungen von `contextra_` in den Quellen. Clippy und Formatierungsprüfungen sauber. |

---

## 2. Abweichungen von SPEC-v2

Keine materiellen Abweichungen. Die Umsetzung folgt strikt SPEC-v2 §4.1, §4.2, §5.1, §5.4 und den Schritten S-10/S-11.
- `ErasureId` und `GateId` verbleiben gemäß SPEC-v2 §4.1 / S-51 als deprecated in `contextra-types::ids` und wurden nicht nach `contextra-ids` verschoben.
- `CollectionId::INVALID` (`CollectionId(0)`) wurde konsistent im `contextra-ids`-Vertrag bereitgestellt.

---

## 3. Koordinationsanforderungen (für Folge-Pakete)

1. **`governance/ratchet.toml`**: `forbid_unsafe_crates` von 30 auf 32 anheben (da `contextra-ids` und `contextra-env` beide `#![forbid(unsafe_code)]` strikt erzwingen).
2. **`capabilities.toml`**: Einträge für `contextra-ids` (Ring 0, dependencies: blake3, serde, thiserror) und `contextra-env` (Ring 0, dependencies: std) nachtragen.
3. **`CODEOWNERS`**: Pfade `crates/contextra-ids/**` und `crates/contextra-env/**` eintragen.
4. **CI-Pfadfilter**: Matrix-Builds und PR-Checks für `contextra-ids` und `contextra-env` aktivieren.

---

## 4. API-Freeze-Tabelle

| Typ / Funktion | Vorheriger Ort | Neuer Ursprungsort | Re-Export / Kompatibilitätspfad | Quellkompatibel |
|---|---|---|---|---|
| `DocId` | `contextra-types` | `contextra-ids` | `contextra_types::DocId` | Ja |
| `EntityId` | `contextra-types` | `contextra-ids` | `contextra_types::EntityId` | Ja |
| `TxId` | `contextra-types` | `contextra-ids` | `contextra_types::TxId` | Ja |
| `TenantId` | `contextra-types` | `contextra-ids` | `contextra_types::TenantId` | Ja |
| `CollectionId` | `contextra-types` | `contextra-ids` | `contextra_types::CollectionId` | Ja |
| `FileId`, `StoreId`, `Seq`, `Timestamp` | `contextra-types` | `contextra-ids` | `contextra_types::{FileId, StoreId, Seq, Timestamp}` | Ja |
| `DocKey`, `DocRef`, `TenantRef`, etc. | `contextra-types` | `contextra-ids` | `contextra_types::{DocKey, DocRef, ...}` | Ja |
| `TenantScoped`, `TenantScopeViolation` | `contextra-types` | `contextra-ids` | `contextra_types::{TenantScoped, TenantScopeViolation}` | Ja |
| `Clock`, `SystemClock`, `unix_ms` | `contextra-ports` | `contextra-env` | `contextra_ports::{Clock, SystemClock, unix_ms}` | Ja |
| `Rng`, `SeededRng` | `contextra-ports` | `contextra-env` | `contextra_ports::{Rng, SeededRng}` | Ja |
| `IdGen`, `SequentialIdGen` | `contextra-ports` | `contextra-env` | `contextra_ports::{IdGen, SequentialIdGen}` | Ja |
| `MetricsSink`, `NoopMetricsSink`, etc. | `contextra-ports` | `contextra-env` | `contextra_ports::{MetricsSink, ...}` | Ja |

---

## 5. Kern-API-Lücken

Keine. Alle vorhandenen Signaturen und Aufrufer bleiben über die Re-Exports und `From<IdError> for ContextraError` ohne Brüche funktionsfähig.

---

## 6. Messwerte Vorher / Nachher

| Kennzahl | Vorher (Baseline) | Nachher (FND-1) | Differenz |
|---|---|---|---|
| Crate `contextra-ids` | Nicht existent | 1 Crate (Ring 0 Mikro-Kern) | +1 Crate |
| Crate `contextra-env` | Nicht existent | 1 Crate (Ring 0 Mikro-Kern) | +1 Crate |
| Interne Deps `contextra-ids` | n/a | 0 (vollkommen autark) | 0 |
| Interne Deps `contextra-env` | n/a | 0 (vollkommen autark) | 0 |
| Unit- & Integrationstests types/ports | Alle grün | Alle grün + 10k Differentialtests | +Differentialtests |

---

## 7. Offene Risiken

Keine offenen Risiken für nachfolgende Pakete. Store, Vector, Graph und Engine können nun konfliktfrei auf `contextra-ids` und `contextra-env` migrieren.
