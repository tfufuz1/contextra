# Spezifikation: `contextra-types`

**Ring:** Ring 0 (Foundation)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §4, §6.3
**Klasse:** B (Storage Bundle Basistypen)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-types` bildet das fundamentale, typentheoretische Rückgrat des Contextra-Ökosystems:
- **Scope:** Kanonische Identifikatoren (`DocId`, `TenantId`, `TxId`, `CollectionId`), Datenstrukturen (`Document`, `KeyBuf`), geschlossene Fehlerdomäne (`ContextraError`) und `RequestContext`.
- **Systemgrenzen:** Rein in-memory, null Netzwerk/Dateisystem-I/O, kein Tokio/Async-Runtime. Vollständige Entkopplung von konkreten Speichertriebwerken.
- **Löschdomänen-Entkopplung:** Spezifische Löschketten- und Saga-Typen (`SagaRecord`, `ScanMethod`, `ErasureId`, `GateId`) sind als `#[deprecated]` markiert und verbleiben nur bis zur vollständigen Auslagerung nach `contextra-erasure-spi`.

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-types`:*
- **LOC:** 5.684 LOC über 25 Rust-Dateien.
- **Typen-Inventar:** 0 Traits, 40 Structs, 15 Enums, 140 öffentliche Funktionen (`pub fn`).
- **Autarkie-Score:** 90/100 (0 interne Workspace-Abhängigkeiten).
- **Zero-Panic-Audit:** 0 unkontrollierte Panics im Produktionscode (`tools/zero_panic_analyzer.py`).

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `ids` | Typsichere Identifikatoren | `DocId` (64/128 Bit), `TenantId`, `TxId`, `CollectionId` |
| `tags` | Deterministische Bit-Tags und Metadaten-Flags | `DocTag`, `CollTag`, `TenantTag` |
| `key` | Fixierte Key-Buffer und Präfix-Codierung | `KeyBuf` (78 Bytes fixiert) |
| `error` | Zentraler Systemfehler-Enum | `ContextraError`, `ErrorDto` |
| `domain` | Dokumenten- und Felddefinitionen | `Document`, `FieldValue`, `Schema` |
| `limits` | Kapazitäts- und Ressourcenbegrenzungen | `ResourceBudget`, `ResourceTracker` |
| `secret` | Zeroize-geschützte Geheimnisträger | `VolatileSecret`, `Sensitive` |

---

## 4. Invarianten (INV-TYP-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-TYP-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird ausnahmslos im gesamten Crate erzwungen. | `cargo check -p contextra-types` |
| `INV-TYP-DETERMINISM-1` | Bit-identische Repräsentationen für alle ID-Typen; deterministische Sortier- und Hash-Ordnung. | `tests/golden_bytes_suite.rs` |
| `INV-TYP-DOCID-WIDTH` | `DocId` unterstützt via Feature `docid-128` nahtlos 128-Bit breite IDs bei vollständiger Wire-Format-Stabilität. | `cargo test -p contextra-types --features docid-128` |
| `INV-TYP-NO-IO` | Der gesamte Crate enthält keinerlei synchrone oder asynchrone I/O-Primitive. | `tools/rust_ast_inspector.py` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. Verlagerung der Kern-IDs nach `contextra-ids`.
2. Übertragung von `tombstone.rs` nach `contextra-mvcc`.
3. Vollständiger Rückbau der Deprecated Re-Exporte in Welle 6.
