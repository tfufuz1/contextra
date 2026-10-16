# Contextra Architecture & Quality Audit Report: `contextra-types`

**Datum:** 2026-03-30
**Crate:** `crates/contextra-types`
**Ring-Ebene:** Ring 0 (Kanonische Domain-Typen & Primitiven)
**Safety Status:** `#![forbid(unsafe_code)]`
**Auditor:** Principal Senior Rust Architect

---

## Executive Summary

`contextra-types` bildet das fundamentale Ring-0-Fundament von Contextra (`DocId`, `TxId`, `TenantId`, `EntityId`, `DistanceMetric`, Filter-AST, Budget-Typen). Ein Audit über alle 8 Architekturprüfpunkte (P1–P8) wurde durchgeführt. Das Crate ist frei von Unsafe-Code, schützt Budget-Arithmetik gegen Überlauf und bewahrt die strikte DAG-Reinheit von Ring 0 (`may_depend_on = []`).

Es wurden jedoch zwei wesentliche Befunde identifiziert:
1. **Filter-AST Fehlen einer Rekursionstiefenbegrenzung (P4):** `FilterExpr::evaluate()` verwendet unbegrenzte Rekursion. Ein synthetischer Stresstest ergab, dass Auswertungen bis 1.000 Ebenen fehlerfrei funktionieren, ab ~10.000 Ebenen jedoch zu einem unkontrollierten **Stack Overflow (SIGABRT)** führen (Schwachstelle für DoS bei extern gespeisten Filtern).
2. **`TenantId` Serialisierungs-Bypass (P2):** Das abgeleitete `#[derive(Deserialize)]` für `TenantId` erlaubt die Deserialisierung von JSON `"0"` zu `TenantId(0)` ohne Validierung durch `try_new()`, was die Invariante `INV-TENANT-1` bei IPC/Storage-Grenzen umgeht. Zudem existiert die als `#[deprecated]` markierte `TenantId::new(u64)` weiterhin im Code.

---

## (1) Prüftabelle P1–P8

| ID | Prüfpunkt | Status | Fundstelle(n) | Befund & Details |
| :--- | :--- | :---: | :--- | :--- |
| **P1** | **Newtype-Disziplin** | ✅ | `ids.rs:11`, `ids.rs:86`, `ids.rs:143/151`, `ids.rs:267`, `ids.rs:364`, `error.rs:39` | Alle Core-ID-Typen (`TenantId`, `CollectionId`, `DocId`, `EntityId`, `TxId`, `StepId`) sind als `#[repr(transparent)]` über `u64` definiert. Unter Feature `docid-128` (ADR-082) ergibt `DocId` korrekt `#[repr(transparent)] pub struct DocId(pub u128)`. `TenantId` und `TxId` verzichten auf ein unvalidiertes `From<u64>`. `TenantId::new()` ist als `#[deprecated]` markiert. |
| **P2** | **INV-TENANT-1 (`TenantId(0)` = SYSTEM)** | ⚠️ | `ids.rs:11-42`, `ids_tests.rs:10-60`, `ids_tests.rs:414-448` | `try_new(0)` gibt `Err(ContextraError::InvalidInput)`, `try_new(1)` gibt `Ok(TenantId(1))`. Unit-Tests decken Grenzfälle ab. **Befund:** Der Standard-`#[derive(Deserialize)]` erlaubt das Umgehen von `try_new()` bei der Deserialisierung von JSON `"0"`. Zudem ist `TenantId::new(0)` trotz Deprecation-Attribut weiter nutzbar. |
| **P3** | **Distanzmetrik** | ✅ | `document.rs:7-115`, `doc_tests.rs:11-247` | Enum `DistanceMetric` trägt `#[non_exhaustive]`. `Cosine`, `Euclidean` und `DotProduct` sind bezüglich Wertebereich, Formel und "smaller = closer"-Semantik (auch in `compute_u8()`) dokumentiert und symmetrisch implementiert. Fließkomma-Clamping (`[0.0, 2.0]`) sichert Cosine-Toleranzen. |
| **P4** | **Filter-AST Tiefenbegrenzung** | ⚠️ | `filter.rs:14-165` | `FilterExpr::evaluate()` wertet Ausdrücke rekursiv aus. Es existiert **keine Prüfinstanz** für die maximale Verschachtelungstiefe. Test-Ergebnis: 1.000 Verschachtelungsebenen werden stabil verarbeitet; bei 10.000 Ebenen kommt es zum Stack Overflow (SIGABRT). |
| **P5** | **Budget-Arithmetik** | ✅ | `budget.rs:188`, `budget.rs:196`, `budget.rs:263`, `budget.rs:370`, `budget.rs:403` | `TokenBudget` und `ResourceTracker` nutzen konsequent `saturating_sub`, `saturating_add` und `checked_add`. Es existiert kein ungesichertes `raw -` / `raw +` auf Budget-Koinzidenzen. |
| **P6** | **Zero-Unsafe** | ✅ | `lib.rs:9` | Modulweites `#![forbid(unsafe_code)]` ist aktiv. Grep nach `unsafe` ergibt exakt 0 Treffer außerhalb des Modul-Attributes. |
| **P7** | **DAG-Reinheit** | ✅ | `Cargo.toml:may_depend_on = []`, `xtask check-ring-layering-full` | Ring 0 Reinheit ist 100% gewahrt. Keine Imports von Ring-1+ Crates (`contextra-types` hängt von keiner Fachcrate ab). |
| **P8** | **Duplikat-Primitiven** | ⚠️ | `xtask check-duplicate-core-primitives` | 3 verstreute Typen-Duplikate in höhergelegenen Crates identifiziert (siehe Kollisionsmatrix). |

---

## (2) Kollisionsmatrix (Typ-Duplikate workspace-weit)

| Typ / Primitive | Duplikat in Crate & Pfad | Schwere | Befund & Empfehlung |
| :--- | :--- | :---: | :--- |
| `HybridQueryBuilder` | `crates/contextra-engine/src/collection/query_builder/builder.rs:23`<br>vs. `crates/contextra-types/src/types/saos.rs:279` | **HOCH** | `contextra-engine` baut eine eigene `HybridQueryBuilder`-Struktur auf, anstatt den kanonischen Type aus Ring 0 (`contextra-types`) zu reexportieren oder wiederzuverwenden. |
| `Edge` | `crates/contextra-graph/src/csr/types.rs:19`<br>vs. `crates/contextra-types/src/types/domain/misc.rs:154` | **MITTEL** | `contextra-graph` verwendet eine spezialisierte CSR-Graph-Kante. Der Typname kollidiert mit der allgemeinen Domain-`Edge` in Ring 0. Umbenennung in `CsrEdge` in Ring 1 empfohlen. |
| `TlHfdParams` | `crates/contextra-graph/src/tl_hfd/params.rs:10`<br>vs. `crates/contextra-types/src/types/domain/misc.rs:346` | **GERING** | Dokumentierter Spiegelungs-Typ in Ring 0 für Ring 1 `contextra-graph` zur Vermeidung zyklischer Crate-Abhängigkeiten (AK-16). Konsistent und beabsichtigt. |

---

## (3) Test-Ergebnis & Coverage

- **Befehl:** `cargo test -p contextra-types --locked -- --nocapture`
- **Ergebnis:**
  - **Unit-Tests (`src/`):** 120 passed, 0 failed, 0 ignored.
  - **Integrationstests (`tests/tenant_scope_isolation.rs`):** 4 passed, 0 failed, 0 ignored.
  - **Gesamt:** 124 passed (100% Erfolg).
- **Line Coverage Threshold (Gate):** 85.0% (Ring 0 / `contextra-core` Gate Benchmark).
- **Abdeckung Status:** Alle Modul-Pfade (ids, document, filter, budget, error, tombstone, saos, importance) verfügen über dedizierte Unit-Tests.

---

## (4) Clippy-Ergebnis & Rustdoc-Qualität

- **Clippy-Befehl:** `cargo clippy -p contextra-types --all-targets --all-features -- -D warnings`
  - **Status:** PASS (0 Warnings, 0 Errors).
- **Rustdoc Missing Docs Check:** `cargo doc -p contextra-types --no-deps 2>&1 | grep "warning: missing" | wc -l`
  - **Status:** 0 missing doc warnings (100% dokumentiert).

---

## (5) VERDICT & SIGN-OFF

**VERDICT:** `CONDITIONAL`

### Vorbedingungen für vollständige Freigabe (Folgeaufträge):
1. **F1 (Filter-AST Tiefenbegrenzung):** Implementierung einer maximalen Verschachtelungstiefe (`MAX_FILTER_DEPTH = 64`) bei der Deserialisierung / Konstruktion von `FilterExpr` oder einer iterativen/tiefenbegrenzten Evaluierung in `FilterExpr::evaluate()`.
2. **F2 (`TenantId` Deserialisierungs-Schutz):** Anbindung von `#[serde(try_from = "u64")]` oder Implementierung eines benutzerdefinierten Serde-Deserializers für `TenantId`, damit JSON `"0"` auch bei der Deserialisierung strikt abgelehnt wird.
3. **F3 (`HybridQueryBuilder` De-Duplizierung):** Konsolidierung des `HybridQueryBuilder` in `contextra-engine` zur Nutzung der Ring-0-Definition.

---

**VERIFIED-BY-SESSION:** PENDING (TS: 2026-09-27T20:15:21Z)
