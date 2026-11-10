# Spezifikation: `contextra-db`

**Ring:** Ring 3 (Orchestration & Cognition)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.3
**Klasse:** C (Integrations- und Kompositionsschicht)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-db` dient als primäre High-Level-Datenbank- und Collection-Schicht für das Contextra-System:
- **Zweck:** Bereitstellung der `Contextra`-Hauptklasse, Collection-Verwaltung, Mehr-Index-Transaktionen und Document-Level CRUD-Operationen.
- **Systemgrenzen:** Kapselt und orchestriert `contextra-engine`, `contextra-store`, `contextra-vector`, `contextra-text` und `contextra-graph` hinter einer konsistenten API.
- **Volatile Vault Integration:** Unterstützung für kurzlebige, verschlüsselte KV-Puffer (`VolatileContextVault`).

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-db`:*
- **LOC:** 1.512 LOC über 5 Rust-Dateien.
- **Typen-Inventar:** 0 eigene Traits, 14 Structs, 2 Enums, 34 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 3 interne Crates (`contextra-types`, `contextra-engine`, `contextra-checkpoint`).
- **Kopplungs- und Leckbefund:** 0 Deletion-Leaks. Hoher Wiederverwendungsgrad über `contextra-engine`.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `lib.rs` | Re-Exporte, Initialisierung und Fassaden-Konfiguration | `Contextra`, `Collection`, `Document` |
| `collection.rs` | Collection-Lebenszyklus, Schema-Validierung, Batch-Operationen | `CollectionHandle`, `CollectionMetadata` |
| `volatile_vault.rs` | Ephemerer Tresor für kurzlebige Kontext-Segmente | `VolatileContextVault`, `VaultConfig` |
| `multistep.rs` | Mehrstufige atomare Transaktionen über Indizes hinweg | `MultiStepTransaction`, `TxContext` |
| `error.rs` | High-Level Datenbank-Fehlerkapselung | `DbError`, `Result<T>` |

---

## 4. Invarianten (INV-DB-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-DB-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird strikt erzwungen. | `cargo check -p contextra-db` |
| `INV-DB-ATOMIC-CRUD-1` | Dokumenten-Operationen werden über alle aktiven Indizes (Vektor, Text, Graph) 2-Phase-Commit-konsistent ausgeführt. | `tests/db_integration.rs` |
| `INV-DB-VAULT-COMMIT-1` | `VolatileContextVault::commit()` liefert deterministische `CommitReceipt` mit Hash-Referenz. | `tests/vault_commit_test.rs` |
| `INV-DB-TENANT-ISOLATION` | Jede Operation erfordert einen validierten `TenantId`; mandantenübergreifende Datenlecks sind strukturell ausgeschlossen. | `tests/tenant_boundary.rs` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. Auslagerung von `volatile_vault.rs` in ein eigenständiges `vault`-Modul / Crate.
2. Anbindung von `pid_latency_controller.rs` und `homeostat.rs` via `contextra-adapt`.
3. Optionale Anbindung von `contextra-cognition` über das Feature `cognition`.
