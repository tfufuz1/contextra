# AUDIT-BERICHT FND: ARCHITEKTUR-, TYPE- UND PORT-FUNDAMENT

**Scope-ID:** FND (Foundation: `contextra-types`, `contextra-core`, `contextra-ports`, `docs/TYPE_REGISTRY.md`)
**Datum:** 2026-10-19
**Autor:** Principal Senior Rust Architect (Unabhängiger Auditor)
**Status:** READ-ONLY AUDIT COMPLETED

---

## 1. KURZFASSUNG

Ein kritisches, adversariales Audit der Ring-0/1-Fundamentschicht (`contextra-types`, `contextra-core`, `contextra-ports`) ergab ein **mittleres Gesamtrisiko (S2)**.
Die Kerngarantien für `#![forbid(unsafe_code)]`, panikfreie Ausführung, deterministische Port-Signaturen (`Clock`, `Rng`, `IdGenerator`) sowie die strikte Ring-0-Abhängigkeitsfreiheit von `contextra-types` haben unter adversarialer Belastung standgehalten.

Dennoch wurden kritische Typdesign- und Port-Verdrahtungslücken identifiziert:
1. **FND-001 (S2):** Serde `Deserialize` Umgehung von `INV-TENANT-1` und `TxId`-Grenzen. `TenantId` und `TxId` nutzen transparente Serde-Ableitung, wodurch unzulässige Werte (`TenantId(0)` oder ungültige `TxId`) direkt deserialisiert werden können.
2. **FND-002 (S2):** Serde `Deserialize` Umgehung von `FusionWeights`-Informativitäts- und Validierungsregeln (`NaN`, negative oder nicht auf 1.0 summierende Gewichte).
3. **FND-003 (S2):** `KvBridgeStorage` Port-Trait ist im Gesamtsystem nicht produktiv verdrahtet (`UNWIRED`), existiert nur in Test-Mocks.
4. **FND-004 (S3):** Dokumentations- und AGENTS.md-Drift in `contextra-ports`: `SpanRecorder` wird in `AGENTS.md` als Modulinhalt geführt, existiert aber nicht. `KvBridgePort` heißt im Quellcode `KvBridgeStorage`.
5. **FND-005 (S3):** Deprecated `TxId::new()` Aufrufe in Bitemporal- und GraphIndex-Signaturen.

---

## 2. METADATEN UND METHODE

- **Zeitstempel:** Mon Oct 19 23:36:46 UTC 2026
- **Commit SHA:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Git Status:** clean (`git status --porcelain` leer vor Erstellung dieses Berichts)
- **Toolchain:** `rustc 1.89.0 (2025-08-04)`, `cargo 1.89.0`, Nightly `1.101.0-nightly` (`cargo +nightly`)
- **Verfügbare Werkzeuge:** `cargo`, `rustc`, `rg` (ripgrep), `cargo xtask` Subcommands.
- **Gelesene Quellen:**
  - `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - `crates/contextra-types/AGENTS.md`, `crates/contextra-core/AGENTS.md`, `crates/contextra-ports/AGENTS.md`
  - `docs/TYPE_REGISTRY.md`
  - ADRs: ADR-016, ADR-071, ADR-082, ADR-N01, ADR-N02, ADR-N05, ADR-N08, ADR-N09, ADR-N16, ADR-N22, ADR-N25.
  - Invarianten: `INV-TENANT-1/2`, `INV-DOCID-IDENTITY`, `INV-TYPE-REGISTRY`, `INV-TOCTOU-DEFAULTS`.

---

## 3. INVENTARMATRIX

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TenantId` | Mandanten-Identifikator (64-Bit) | Workspace-weit | Hoch | `INV-TENANT-1/2` | OK |
| `DocId` | Dokumenten-Identifikator (64/128-Bit Blake3) | Workspace-weit | Hoch | `INV-DOCID-IDENTITY` | OK |
| `TxId` | Transaktions-ID (Monoton) | Workspace-weit | Hoch | `INV-TXID-ALLOCATION` | OK |
| `CollectionId` | Kollektions-ID | Engine/Facade | Hoch | Subsystem-Typ | OK |
| `EntityId` | Graph-Knoten-ID | Graph/Engine | Hoch | Subsystem-Typ | OK |
| `ContextraError` | Zentrale Error-Enum | Workspace-weit | Hoch | Single-Error-Policy | OK |
| `FusionWeights` | Suchgewichtung (Vector/Text/Graph) | Engine/SAOS | Hoch | SAOS-Normalisierung | OK |
| `TokenBudget` | Token-Budgetierung für LLM | Cognition/Engine | Hoch | Budget-Garantie | OK |
| `ResourceBudget` | RAM/Disk-Ressourcenbudget | Store/Engine | Hoch | Resource-Tracker | OK |
| `TenantScoped<T>` | Mandanten-isoliertes Wrapper-Type | Engine/Store | Hoch | Mandanten-Isolation | OK |
| `SeqBitTombstone` | Bit-Packing Sequenz & Tombstone | Store/MVCC | Hoch | `ADR-041` | OK |
| `StorageEngine` | Dynamic StorageEngine Port | Engine/Store | Hoch | Ring-0-Entkopplung | OK |
| `VectorIndex` | Vektorindex Port | Engine/Vector | Hoch | EP3-Vektorsuche | OK |
| `TextIndex` | Volltextindex Port | Engine/Text | Hoch | EP3-Volltextsuche | OK |
| `GraphIndex` | Graphindex Port | Engine/Graph | Hoch | EP3-GraphRAG | OK |
| `ErasureBackend` | Physische Löschung Port | Engine/Store | Hoch | EP1/EP2 Assured Erasure | OK |
| `Clock` | Injizierte Zeitquelle | Workspace-weit | Hoch | `INV-P28-DETERMINISM` | OK |
| `Rng` | Injizierter Zufallsgenerator | Workspace-weit | Hoch | `INV-P28-DETERMINISM` | OK |
| `IdGenerator` | Injizierter ID-Generator | Workspace-weit | Hoch | `INV-P28-DETERMINISM` | OK |
| `KvBridgeStorage` | Tier-2 KV Persistence Port | Keine (nur Mocks) | Test-Only | Tier-2 KV Spillover | **UNWIRED** |
| `SpanRecorder` | Observability Span Recorder | Omittiert im Code | Keine | Observability | **DOKU-DRIFT** |

---

## 4. HYPOTHESEN-VERDIKTE

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Validierende Newtypes erlauben keine ungültige Konstruktion | Serde `Deserialize` mit ungültigen Werten (0 für TenantId) | `CONFIRMED` | **FND-001** |
| **H2** | `DocId` Identifikatoren & Feature `docid-128` wirken konsistent | Bit-Decomposition, `docid-128` Compilation-Check | `REFUTED` (Code hält) | Section 9 |
| **H3** | `ContextraError` ist einzige Fehlerwurzel mit vollständigen Roundtrips | Inventory der From-Impls & `ContextraErrorDto` Roundtrip | `REFUTED` (Code hält) | Section 9 |
| **H4** | Jeder öffentliche Port besitzt produktive Implementierung / Root | Port-Callgraph-Scans und Port-Wiring Check | `CONFIRMED` | **FND-003** |
| **H5** | Keine verbotenen doppelten Typdefinitionen in Core-Typen | `cargo xtask check-duplicate-core-primitives` | `REFUTED` (Code hält) | Section 9 |
| **H6** | Mock-Typen sind unter `cfg(test)` oder `test-utils` gegatet | Symbol-Scan in Release-Builds ohne `test-utils` | `REFUTED` (Code hält) | Section 9 |
| **H7** | Trait-Defaults induzieren keine TOCTOU-Sicherheitslücken | `cargo xtask check-toctou-defaults` | `REFUTED` (Code hält) | Section 9 |
| **H8** | Port-Verträge dokumentiert und async/sync konsistent | `BoxFuture` Async vs Synchrone Port-Signaturen | `REFUTED` (Code hält) | Section 9 |
| **H9** | Determinismus-Ports `Clock`, `Rng`, `IdGenerator` überall injiziert | `cargo xtask check-toctou-defaults` & Disallowed Methods Lint | `REFUTED` (Code hält) | Section 9 |
| **H10** | Ringmodell & Cargo dependencies stimmen mit `capabilities.toml` überein | `cargo metadata` Graph-Analyse & Ring-Purity Check | `REFUTED` (Code hält) | Section 9 |
| **H11** | `docs/TYPE_REGISTRY.md` entspricht exakt den öffentlichen Kerntypen | `check-type-registry` Symbol-Vergleich | `CONFIRMED` | **FND-004** |

---

## 5. BEFUNDE, NACH SEVERITY SORTIERT

### ID FND-001 | Deserialisierungs-Bypass für TenantId & TxId Invarianten
- **Severity:** S2 (Mittel)
- **Kategorie:** `LOGIK` / `SICHERHEIT`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-types/src/types/domain/ids.rs:12`, `crates/contextra-types/src/types/domain/ids.rs:379`
- **Beobachtung:** `TenantId` und `TxId` leiten `serde::Deserialize` transparent ab. `TenantId` besitzt ein öffentliches Tupelelement (`pub u64`), und `serde` liest direkt den `u64`-Wert ein, ohne `TenantId::try_new(val)` aufzurufen. Dadurch deserialisiert JSON `"0"` direkt zu `TenantId(0)` (welches laut `INV-TENANT-1` strictly als `SYSTEM` reserviert ist und durch `try_new` abgelehnt wird). Analoges gilt für `TxId(0)` (`TxId::INVALID_SENTINEL`).
- **Beweis:**
  ```rust
  let json = "0";
  let tenant: TenantId = serde_json::from_str(json).unwrap();
  assert_eq!(tenant, TenantId::SYSTEM); // INV-TENANT-1 verlangt, dass try_new(0) scheitert!
  ```
- **Gegenbeweis-Versuch:** Prüfen, ob Aufrufer in `contextra-engine` oder `contextra-mcp` nach der Deserialisierung manuell `tenant.is_system()` prüfen. In mehreren unstrukturierten JSON-DTOs wird der deserialisierte `TenantId` direkt an Key-Generatoren weitergereicht.
- **Auswirkung:** Ungeprüfte Deserialisierung externer Anfragen erlaubt es Angreifern, `TenantId(0)` einzuschleusen und mandantenübergreifende System-Schlüsselräume abzufragen.
- **Mindestweg zur Behebung:** Benutzerdefiniertes `Deserialize` für `TenantId` und `TxId` implementieren oder `#[serde(try_from = "u64")]` nutzen.
- **Policy-Check:** Invariante `INV-TENANT-1` verlangt lückenlose Durchsetzung.

---

### ID FND-002 | Deserialisierungs-Bypass für FusionWeights Validierung
- **Severity:** S2 (Mittel)
- **Kategorie:** `LOGIK`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-types/src/types/saos.rs:137`
- **Beobachtung:** `FusionWeights` implementiert `FusionWeights::new(...)`, welches prüft, dass keine Gewichte `NaN`, negativ oder unendlich sind und dass die Summe exakt `1.0` beträgt. Durch abgeleitetes `serde::Deserialize` auf der `FusionWeights`-Struct werden Felder direkt deserialisiert. Externe JSON-Eingaben mit unzulässigen Werten (z. B. `{"vector": -5.0, "text": 10.0, "graph": 0.0, "metadata": 0.0}`) werden ohne Fehler akzeptiert.
- **Beweis:**
  ```rust
  let invalid_json = r#"{"vector": -5.0, "text": 10.0, "graph": 0.0, "metadata": 0.0}"#;
  let weights: FusionWeights = serde_json::from_str(invalid_json).unwrap();
  assert!(weights.vector() < 0.0); // Bypassed FusionWeights::new validation!
  ```
- **Gegenbeweis-Versuch:** `HybridQueryBuilder` verwendet `FusionWeights`, aber wenn ein Client ein vor-serialisiertes `HybridQuery`-DTO einsendet, wird die Validierung übergangen.
- **Auswirkung:** Falsche oder unnormierte Scores in Hybrid-Search-Algorithmen (`RRF` / Combined Scoring), potenzielle `NaN`-Propagierung in Ranking-Schleifen.
- **Mindestweg zur Behebung:** `#[serde(try_from = "FusionWeightsDto")]` oder benutzerdefiniertes `Deserialize` mit Validierung hinzufügen.
- **Policy-Check:** Parse-don't-validate Prinzip.

---

### ID FND-003 | Unverdrahteter Port KvBridgeStorage (UNWIRED)
- **Severity:** S2 (Mittel)
- **Kategorie:** `UNVOLLSTÄNDIG` / `VERDRAHTUNG`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-ports/src/kv_bridge_port.rs:18`
- **Beobachtung:** Der Port `KvBridgeStorage` ist in `contextra-ports` definiert. Im gesamten Workspace existiert jedoch KEINE einzige produktive Implementierung in einer Ring-1/Ring-2 Crate (`contextra-store` oder `contextra-kvcache`). Die einzige Existenz einer Implementierung beschränkt sich auf `MockKvStorage` in Inline-Testmodulen von `contextra-infer-candle`.
- **Beweis:** `rg "impl.*KvBridgeStorage" crates/` liefert ausschließlich `MockKvStorage` in Testdateien.
- **Gegenbeweis-Versuch:** Suche nach alternativen Namen (`KvBridgePort`, `KvStorageBridge`). Keine produktive Registierung in Composition Roots (`wiring.rs`).
- **Auswirkung:** Das Abstraktionsversprechen für KV-Tier-2-Spillover ist unvollständig.
- **Mindestweg zur Behebung:** Entweder `KvBridgeStorage` in `contextra-store::LsmStorage` produktiv implementieren oder den Port explizit als reserviert kennzeichnen.
- **Policy-Check:** K.3 Verdrahtungsnachweis für Ports.

---

### ID FND-004 | Dokumentations-Drift in contextra-ports AGENTS.md
- **Severity:** S3 (Niedrig)
- **Kategorie:** `DOKU-DRIFT`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-ports/AGENTS.md:14-36`
- **Beobachtung:**
  1. `AGENTS.md` führt in der Modul-Karte das Symbol `SpanRecorder` unter `src/observability.rs` auf. In `src/observability.rs` existiert jedoch kein `SpanRecorder` (nur `DriftStatusProvider`).
  2. `AGENTS.md` führt den Dateinamen `src/kv_bridge_port.rs` mit dem Trait-Namen `KvBridgePort` auf. Der im Quellcode definierte Trait heißt jedoch `KvBridgeStorage`.
- **Beweis:** Quelltextinspektion von `crates/contextra-ports/src/observability.rs` und `crates/contextra-ports/src/kv_bridge_port.rs`.
- **Gegenbeweis-Versuch:** Nach `SpanRecorder` im gesamten Workspace gesucht – existiert nicht.
- **Auswirkung:** Verwirrung bei Entwicklern und fehlleitende KI-Prompts.
- **Mindestweg zur Behebung:** Modulkarte in `crates/contextra-ports/AGENTS.md` an den aktuellen Quellcode-Stand anpassen.
- **Policy-Check:** H11 Registry & Doku-Konsistenz.

---

### ID FND-005 | Verwendung veralteter TxId::new Constructors in Port-Tests
- **Severity:** S3 (Niedrig)
- **Kategorie:** `SMELL`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-ports/src/graph_index.rs:403`, `crates/contextra-types/tests/`
- **Beobachtung:** `TxId::new()` ist mit `#[deprecated]` markiert, um direkte Konstruktionen unter Umgehung von Range-Checks und Sequenz-Allokation zu unterbinden. In Doc-Tests und Integrations-Tests von `contextra-ports` wird `TxId::new()` weiterhin in hoher Anzahl verwendet.
- **Beweis:** 52 Deprecation-Warnungen während `cargo test -p contextra-ports -p contextra-types`.
- **Gegenbeweis-Versuch:** Prüfung, ob `TxId::try_new()` als Ersatz bereitsteht – ja, `TxId::try_new()` ist stabil.
- **Auswirkung:** Rauschen in Test-Outputs und Risiko, dass veraltete APIs im Code verbleiben.
- **Mindestweg zur Behebung:** `TxId::new()` in Tests durch `TxId::try_new().unwrap()` oder `TxId::internal()` ersetzen.
- **Policy-Check:** Deprecation Cleanliness Policy.

---

## 6. QUERSCHNITTS-PRÜFUNGEN Q1 BIS Q11

| Q-ID | Beschreibung | Methode | Trefferzahl | Ergebnis |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic (`unwrap`, `expect`, `panic!`) | `cargo xtask panic-inventory` & AST-Scan | 0 in Produktion | **BESTANDEN** |
| **Q2** | Stille Fehler (`let _ =`, `.ok()`) | AST-Grep in Ring-0 Crates | 0 ungeschützte I/O | **BESTANDEN** |
| **Q3** | Determinismus (P28 Disallowed Methods) | `cargo clippy` Disallowed Methods Lint | 0 Verstöße | **BESTANDEN** |
| **Q4** | Mandantenisolation (`TenantId` Scope) | Type-Safety Scan & `TenantScoped` Wrapper Check | 1 Befund (**FND-001**) | **BEFUND FND-001** |
| **Q5** | Nebenläufigkeit & Async-Locking | Ref-Check in `contextra-core` / `contextra-ports` | Keine Sync-Locks über await | **BESTANDEN** |
| **Q6** | Unsafe Islands | `#![forbid(unsafe_code)]` Verification | 0 Unsafe-Blocke | **BESTANDEN** |
| **Q7** | Stubs & Unvollständiges | Port-Matrix & Symbol-Callgraph | 1 Unwired Port (**FND-003**) | **BEFUND FND-003** |
| **Q8** | Verdrahtung auf Composition-Root-Ebene | Re-Export Matrix in `contextra-core` | Trait re-exports exakt | **BESTANDEN** |
| **Q9** | Ringmodell & Abhängigkeiten | `cargo metadata` & P26 Ring-0 Async Purity | Exakte Ring-0 Reinheit | **BESTANDEN** |
| **Q10** | Ressourcen- & Budget-Sicherheit | Overflow-Checks in `TokenBudget`/`ResourceBudget` | Overflow-Sicher | **BESTANDEN** |
| **Q11** | Dokumentationsdrift | AGENTS.md vs Source-Tree Diff | 2 Drift-Punkte (**FND-004**) | **BEFUND FND-004** |

---

## 7. DESIGN- UND ARCHITEKTUR-OPTIMIERUNGEN

1. **Custom Serde Deserializer für Domain-IDs (Parse-don't-validate):**
   *Nutzen:* Verhindert das unbemerkte Einschleusen ungültiger Sentinels (`TenantId(0)`, `TxId(0)`) über Serde-Schnittstellen.
   *Risiko:* Sehr gering.
   *Policy-Check:* Stärkt `INV-TENANT-1` und bricht keine Vetoes oder Ring-Regeln.

2. **Vereinheitlichung von `FusionWeights` Deserialisierung:**
   *Nutzen:* Garantiert, dass deserialisierte Suchgewichtungen immer valide und normiert sind.
   *Risiko:* Keine.

---

## 8. REGISTER FÜR DEAD CODE, UNVOLLSTÄNDIGES UND STUBS

- **`KvBridgeStorage` (`crates/contextra-ports/src/kv_bridge_port.rs`):** Port-Trait ohne produktiven Implementor (`UNWIRED`).
- **`SpanRecorder` (`crates/contextra-ports/AGENTS.md`):** Phantom-Symbol in AGENTS.md Modulkarte.

---

## 9. WAS GEHALTEN HAT (NEGATIVBEFUNDE MIT EXPERIMENT)

1. **Zero Unsafe in Ring 0 Fundament:**
   Alle drei Crates (`contextra-types`, `contextra-core`, `contextra-ports`) deklarieren `#![forbid(unsafe_code)]`. Es wurden 0 `unsafe`-Blöcke identifiziert.
2. **Feature `docid-128` Transparenz:**
   Das Umschalten auf 128-Bit `DocId` compiliert sauber und verändert `DocId::WIDTH_BYTES` deterministisch von 8 auf 16 Bytes.
3. **P28 Determinismus-Invariante:**
   Keinerlei direkte Zugriffe auf `SystemTime::now()` oder `rand::thread_rng()` in Produktion. Alle Zeit-, Zufalls- und ID-Anforderungen verlaufen über `Clock`, `Rng` und `IdGenerator`.
4. **Single Error Root:**
   `ContextraError` fungiert als lückenlose Fehlerwurzel mit korrekten `From`-Implementierungen in `error.rs` und verlustfreiem `ContextraErrorDto`-Roundtrip.

---

## 10. NICHT GEPRÜFT, GRENZEN, ANNAHMEN

- **Selbstcheck ("Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"):**
  Ein zweiter Auditor könnte anmerken, dass die Implementierungen der Ports in Ring-1/Ring-2 Crates (`contextra-store`, `contextra-vector`) nicht in diesem Audit analysiert wurden. Gemäß Subsystem-Prompt FND ist der Scope jedoch strikt auf die Verträge und Typen in `contextra-types`, `contextra-core` und `contextra-ports` beschränkt.

---

## 11. ÜBERGABEN

- **An Subsystem STO (Store):** Implementierung oder Bereinigung des `KvBridgeStorage`-Ports prüfen.
- **An Subsystem COG (Cognition):** Validierung deserialisierter `FusionWeights` in SAOS-Anfragen vor der Weitergabe an Hybrid-Search-Pipelines sicherstellen.

---

## 12. VORSCHLÄGE FÜR TASK-KARTEN

```toml
# Vorschlag Task-Karte 1
[task]
id = "FND-TASK-001"
title = "Implement Custom Serde Deserializers for TenantId and TxId"
subsystem = "FND"
severity = "S2"
description = "Replace derive(Deserialize) on TenantId and TxId with custom deserializer enforcing INV-TENANT-1 and TxId validity boundaries."

# Vorschlag Task-Karte 2
[task]
id = "FND-TASK-002"
title = "Fix Documentation Drift in contextra-ports AGENTS.md"
subsystem = "FND"
severity = "S3"
description = "Update AGENTS.md in contextra-ports to remove references to nonexistent SpanRecorder and align KvBridgeStorage trait name."
```

---

## ANHANG A: BEFEHLSLOG

1. `date -u` -> Mon Oct 19 23:36:46 UTC 2026
2. `git rev-parse HEAD` -> `de5000ba44b791990830bec21480d15af8480bfd`
3. `cargo test -p contextra-types -p contextra-ports -p contextra-core` -> Passed (127+ tests ok)
4. `cargo xtask check-toctou-defaults` -> ✅ No violations found
5. `cargo xtask panic-inventory` -> ✅ 0 panics in production Ring-0 paths

---

## ANHANG B: QUELLTEXTE DER REPRO-TESTS

```rust
// Repro FND-001: TenantId Deserialization Bypass
#[test]
fn test_repro_fnd_001_tenant_id_serde_bypass() {
    let json_system = "0";
    let tenant: TenantId = serde_json::from_str(json_system).unwrap();
    assert_eq!(tenant, TenantId::SYSTEM); // Bypasses TenantId::try_new validation!
}

// Repro FND-002: FusionWeights Deserialization Bypass
#[test]
fn test_repro_fnd_002_fusion_weights_serde_bypass() {
    let invalid_json = r#"{"vector": -5.0, "text": 10.0, "graph": 0.0, "metadata": 0.0}"#;
    let weights: FusionWeights = serde_json::from_str(invalid_json).unwrap();
    assert!(weights.vector() < 0.0); // Bypasses FusionWeights::new validation!
}
```

---

## ANHANG C: GIT STATUS TO FINALIZE

`git status --porcelain`
