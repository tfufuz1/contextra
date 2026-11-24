# SP-FACADE — Öffentliche Storage-API, Konfiguration und Observability

| | |
|---|---|
| Modul | `lib.rs`, `config.rs`, `spaces.rs`, `metrics.rs` |
| Besitzt | `Storage`-Fassade, Builder/Config validation, stabile Re-Exports, Metrik-/Observer-Katalog |
| Konsumiert | alle internen SPs; ist oberste Kernschicht |
| Spec-Bezug | R8.2 §3, §4.7, §16; Z-16…Z-18; E-35, E-40, E-41, E-44, E-74…E-81 |
| Phase | P1 Basis bis P8 (G-P) |

## 1 Auftrag
Fassade bindet Komponenten und validiert Konfiguration vor jeder Ressource/Lock/Dateiöffnung. Keine Geschäftslogik duplizieren; Typen/Trait-Signaturen gehören PORTS. Internals nicht versehentlich öffentlich machen.

## 2 API
`Storage::open/close/apply/view/view_at/view_durable/durable_barrier`, `clusters/spaces`, `eraser`, `consumers/segments`, `checkpoint/restore`. Jede Methode delegiert an ihren Besitzer-SP. Konfigurationsfelder mit gefährlichen Defaults (u. a. `purge_deadline`, Anchor-Voraussetzung) sind erforderlich, nicht stillschweigend defaultbar.

## 3 Validierung und Metriken
Validiert Größenkette aus E-35, `DataClass::Personal ⇒ clustered + Encrypted + (Durable ⇒ SealPolicy::Cluster)`, Anchor-Pflicht bei Authenticated/Encrypted, offene Dateien/Reserve/Erase-Parameter. Datenklassifikation ist eine explizite Enum-Deklaration und kein PII-Detektor. Metrikkatalog enthält Namen, Typ, Einheit, erlaubte Labels und Owner-SP; Label-Allowlist ist endlich und wird vor Export validiert. Keine IDs, Schlüssel, Nutzdaten oder dynamische Nutzerlabels. Gate, KeyDestroy und Purge werden getrennt nach Zeit, Ergebnis und Bytes beobachtet. `Error::hint()` ist stabil und inhaltsfrei; `suggested_call` ist optionaler Fassadenvertrag. Inspector zeigt standardmäßig nur Capability-geschützte aggregierte/opaque Diagnosewerte.

## 4 Invarianten / Umsetzung
INV-LIM-1, INV-SEAL-1, INV-ANCH-1. Config table tests; public API doctests; Observer nicht unter Locks und nicht blockierend; Debug/Error Redaction.

## 5 Arbeitspakete
WP-23.1 Config/open skeleton (P1) · WP-23.2 komplette Typisierte API (P3–P6) · WP-23.3 Metrikkatalog + Label-Validator (P6) · WP-23.4 stabiler API-/Doc-Freeze (P8). Abnahme: keine Fassade ohne Besitzer-SP und Configfehler vor I/O.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §3.5, §9.4, §9.5.2, §9.7, §10.4, §11.4, §16; E-74…E-77, E-81.

- **Config:** `OpenProfile::{Development, Production}` (Default `Production` bei persistentem Open), `view_lease`, `view_lease_max`, `tomb_epoch_rotation`, `backup_horizon` (Pflicht, sobald Retire aktiv), `max_active_tomb_epochs`, `acknowledged_weak_keys`, `ledger_same_device_ack`. Validierung vor I/O; `Production` prüft `ProviderAssuranceInsufficient` und `LedgerSameDevice`, `Development` warnt.
- **API:** `Storage::erase_epoch() -> u64`, `Storage::backup_mark() -> BackupMark`, `View::renew()`, Poisoned-Erase-Pfad (Aufnahme über Ledger, §9.5.2), `rotate_tomb_epoch`/`retire_tomb_epoch` als administrative, capability-geschützte Aufrufe.
- **Fehler:** `SnapshotTooOld`, `ErasePending`, `LedgerUnavailable`, `LedgerSameDevice`, `BackupBeyondHorizon`, `ProviderAssuranceInsufficient` mit `hint()`.
- **Metriken (geschlossene Labels):** abgelaufene View-Leases, offene Poisoned-Löschabsichten, Anzahl/Alter aktiver Tomb-Epochen; keine IDs.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-23.5 | R8.2-Fassade: Config/`OpenProfile`, `erase_epoch`, `backup_mark`, `renew`, Poisoned-Erase-Pfad, Fehler, Metriken | 1.5 | WP-23.2, WP-13.7, WP-20.4 | P6 | Config-Validierungstests, Leakage-/Kardinalitätstest der neuen Metriken, `hint()`-Katalogtest |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

`Config::validate` prüft vor I/O die gemeinsamen RAM-, Cluster-Metadaten-, Datei-/FD- und Purge-Scratch-Budgets, `max_live_files`, ClusterLocal-Admission sowie die konsistenten harten Limits `max_body_bytes = 4 MiB`, `max_change_bytes = 8 MiB` und `max_body_bytes ≤ max_change_bytes / 2`. Metriken unterscheiden Stufe-A-Konflikte, `GuardRetry` und Fensterfuß-Treffer; einen Sequencer-„Slowpath“ gibt es nicht. Alle neuen Leistungswerte bleiben Hypothesen bis zum Gate-Nachweis.
