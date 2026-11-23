# SP-FACADE — Öffentliche Storage-API, Konfiguration und Observability

| | |
|---|---|
| Modul | `lib.rs`, `config.rs`, `spaces.rs`, `metrics.rs` |
| Besitzt | `Storage`-Fassade, Builder/Config validation, stabile Re-Exports, Metrik-/Observer-Katalog |
| Konsumiert | alle internen SPs; ist oberste Kernschicht |
| Spec-Bezug | R8.1 §3, §4.7, §16; Z-16…Z-18; E-35, E-40, E-41, E-44 |
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
