# contextra-types — Auflösung und Verteilungsplan (ADR-33, REM-30)

**Ebene:** entfällt. Das Crate wird aufgelöst und verschwindet.
**Status:** Normativ (Soll-Zustand), ersetzt die Fassung 2.1, die das Crate als Kern-Crate beschrieb.
**Version:** 3.0 (SPEC 4.1)
**Geltungsbereich:** `crates/contextra-types` (Quelle), Zielcrates laut Abschnitt 3
**Normative Quellen:** `docs/SPEC.md` ADR-32, ADR-33, ADR-52, Befund F-08, Entfernung REM-19, REM-24, REM-27, REM-30, Gate G-07; Regeln R2, R4, R9.

---

## 1. Ausgangslage und Ziel

**Ist (F-08):** 40 Quelldateien, 9.596 Zeilen. Querschnitts-Crate ohne Besitzer. Es enthält Fehler, IDs, DTOs, Budgets, Suchtypen, Agentengedächtnis, Erasure-Altmodell und Konfiguration in einem Paket. `lib.rs` re-exportiert 16 Gruppen (Zeilen 61–76). `types/ids.rs` ist `pub use contextra_ids::*`.

**Verbraucher:** 266 Dateien importieren `contextra_types`. Die meisten Importe stammen aus `contextra-ports` (201 Dateien; diese Importe fallen durch `ports`-PLAN E-PRT-01 weg), `contextra-vector`, `contextra-text`, `contextra-store`, `contextra-graph`, `contextra-rank`, `contextra-checkpoint`, `contextra-crypto`, `contextra-simd`, `contextra-mvcc`. Dazu kommen `contextra-testkit` und die Fassade `contextra`.

**Ziel:** Kein Querschnitts-Crate. Jeder Typ hat genau einen Besitzer (R4) oder entfällt. Ein Typ, den zwei oder mehr E1-Crates gemeinsam nutzen, wandert nach `contextra-ports` (E0), weil R2 eine Abhängigkeit zwischen E1-Crates verbietet.

**Nicht-Ziel:** Keine inhaltliche Neuerfindung von Typen. Wo die Umbauschritte etwas ändern (Methodensignaturen, Zeitfelder), steht es in Abschnitt 5.

---

## 2. Verteilungsregeln

Die Regeln gelten in dieser Reihenfolge. Die erste zutreffende Regel bestimmt die Aktion.

| Regel | Bedingung | Aktion |
|---|---|---|
| V1 | Begriff ist Altmodell oder durch SPEC 4.1 ersetzt (Agentengedächtnis, Tombstone, Erasure-Altmodell, Schema-Breite) | **entfernen** |
| V2 | Typ ist ein DTO aus SPEC 15 oder ein Typ, den ein Port-Trait in seiner Signatur braucht | **`contextra-ports`** (Modul laut Tabelle) |
| V3 | Typ wird von zwei oder mehr E1-Crates genutzt (R2) | **`contextra-ports`** (Modul laut Tabelle) oder bei reinen Laufzeitzählern **`contextra-env`** |
| V4 | Typ wird nur von einem Crate genutzt | **dieses Crate** (Besitzer) |
| V5 | Typ ist eine Kennung, ein Ordinal oder ein Limit | **`contextra-ids`** (ADR-52: ein Limit, eine Definition) |
| V6 | Typ hat Methoden, die Algorithmen eines anderen Crates brauchen | Besitzer bleibt, Methode wird **freie Funktion** im Zielcrate (Rust erlaubt keine inherenten Methoden über Crate-Grenzen) |
| V7 | Fehler | `contextra-ports::error` für `ContextraError`, Crate-eigene Fehler im Besitzer |
| V8 | Ohne Verbraucher außerhalb des Crates und ohne SPEC-Bezug | **entfernen** (Prüfung in U-TYP-0) |

Eine Zuordnung mit dem Vermerk **„Verbraucher prüfen“** wird in U-TYP-0 durch Zählen festgelegt. Ohne Verbraucher gilt V8.

---

## 3. Verteilung je Quelle

### 3.1 Re-Exporte in `lib.rs` (Zeilen 61–76)

| Re-Export | Ziel | Aktion | Begründung |
|---|---|---|---|
| `ids::ErasureId` | `contextra-ids::TicketId` | umbenennen | E-IDS-11, SPEC 7.2 |
| `ids::GateId` | entfällt | entfernen | E-IDS-12, Gate ist mandantenweit (ADR-28) |
| `ids::FileId`, `ids::StoreId` | `contextra-ids::SegOrd` | ersetzen | E-IDS-09 |
| `ids::Seq` | `u64` in `contextra-ports::version` | ersetzen | E-IDS-07 |
| `ids::Timestamp` | `u64` (Nanosekunden) aus `Clock` in `contextra-env` | ersetzen | E-IDS-08, R10 |
| `refs::CollectionRef`, `refs::TenantRef` | `contextra-ids::CollectionId`, `TenantId` | ersetzen | Sind durch Kennungen ersetzt |
| `refs::DocKey` | `(TenantId, CollectionId, DocId)` im Aufrufer | entfernen | E-IDS-10 |
| `refs::OperatorRef` | `contextra-ids::PrincipalRef` | ersetzen | E-IDS-13, ADR-38 |
| `refs::SubjectRef` | `contextra-ids` (Modul `refs`) | verschieben | Zeroize-Typ laut ids-PLAN, ADR-38 |
| `refs::MAX_REF_LEN` | `contextra-ids::limits` | verschieben | ids-PLAN, Limits |
| `error::ContextraError`, `error::Result` | `contextra-ports::error` | verschieben | V2, 111 Verbraucher |
| `error::HnswDeletionError` | `contextra-vector::error` | verschieben | V4 |
| `error_dto::ContextraErrorDto` | `contextra-mcp::error_payload` | verschieben | nur MCP-Antwortformat, E3 |
| `model_fingerprint::ModelFingerprint` | `contextra-ports::embedding` | verschieben | V3: `crypto` (9 Dateien) und `vector` nutzen es |
| `retrieval_strategy::RetrievalStrategy` | `contextra-ports::dto` | verschieben | V2: `ports` und `rank` |
| `schema::DocIdWidth` | entfällt | entfernen | E-IDS-01, Breite ist fest, 0 Verbraucher |
| `schema::ManifestSchemaVersion` | entfällt | entfernen | 0 Verbraucher; Manifest-Version gehört in `contextra-store`, falls benötigt |
| `tenant_scope::TenantScoped`, `TenantScopeViolation` | entfällt | entfernen | E-IDS-16, Mandantenbindung über Schlüsselwahl und `TenantId` |
| `tombstone::SeqBitTombstone`, `TombstoneSemanticsCheck` | entfällt | entfernen | E-IDS-04, REM-12 |
| `types::*` | siehe 3.2 bis 3.6 | — | — |
| `witness::RequestContext` | `contextra-ports::context` | verschieben | V3: 7 Dateien in `ports`, dazu `text`, `vector`, `crypto`, `graph` |
| `witness::SagaRecord` | `contextra-ports::erasure` | verschieben | `crypto` darf nicht von `erasure` abhängen (ADR-32). Der Typ wird deshalb ein DTO im Port |
| `witness::ScanMethod` | `contextra-ports::erasure` | verschieben | V2, `ports` nutzt ihn in 2 Dateien |

### 3.2 `types/budget.rs` (709 Zeilen, 23 Items)

| Item | Ziel | Aktion |
|---|---|---|
| `TokenBudget`, `BudgetStrategy`, `Reservation` inklusive Methoden (`new`, `for_model`, `with_reserved`, `with_strategy`, `effective_limit`, `available`, `consume`, `reserve`, `try_reserve`, `refund`, `consumed`, `settle`) | `contextra-ports::dto` | verschieben. Rein wertbasiert, keine I/O. Verbraucher: `ports` (3), `testkit` (4) |
| `ResourceBudget`, `ResourceTracker` inklusive `consume_memory`, `release_memory`, `memory_used`, `budget`, `has_memory_capacity` | `contextra-env::budget` | verschieben. Verbraucher: `store` (6), `graph` (4). Beide liegen auf E1, R2 verbietet die Kante zwischen ihnen. `env` ist bereits Abhängigkeit beider. Entscheidung O-TYP-1 |

### 3.3 `types/domain/document.rs` (247 Zeilen, 12 Items)

| Item | Ziel | Aktion |
|---|---|---|
| `DistanceMetric` | `contextra-ports::vector_index` | verschieben. V3: `vector` (87 Dateien) und `simd` (10 Dateien). Der Kern `simd` erhält keine Metrik-Enum (siehe `contextra-simd/PLAN.md`) |
| `DistanceMetric::compute`, `compute_u8` | `contextra-vector::metric::distance`, `distance_u8` | freie Funktionen nach V6. Die Kernel kommen aus `contextra-simd` |
| `Embedding` inklusive `new`, `dim`, `as_slice`, `l2_norm`, `normalize` | `contextra-ports::embedding` | verschieben. V3: `vector` (8), `ports` (3), `testkit` (2), `graph` (1) |
| `ScoredDocument` inklusive `new` | `contextra-ports::dto` | verschieben. V3: `vector` (17), `text` (4), `ports` (2), `testkit` (3) |
| `RerankResult` | `contextra-ports::reranker` | verschieben. Rückgabe des `Reranker`-Ports |

### 3.4 `types/domain/misc.rs` (624 Zeilen, 33 Items)

| Item | Ziel | Aktion |
|---|---|---|
| `Entity`, `Edge` inklusive `new`, `try_new`, `with_source_doc_id`, `with_weight`, `with_validity`, `with_tx_validity`, `with_business_validity` | `contextra-graph::model` | verschieben. Verbraucher: `graph` (56 und 42 Dateien), `testkit` (6 und 5). Zeitfelder werden `u64` (siehe 5, U-TYP-6). E-IDS-05: `EntityId` ist bereits `graph::EntityId` |
| `MAX_SEARCH_K` | `contextra-ids::limits` | verschieben. ADR-52: einziger Wert 1.000, die anderen fünf Definitionen entfallen |
| `LinkRelation`, `MemoryLink`, `MemoryType`, `WorkflowState`, `EXPIRY_METADATA_KEY`, `default_decay`, `default_ttl_tx`, `as_metadata_key`, `serialize`, `deserialize` | entfällt | entfernen nach REM-24 (Agentengedächtnis). Vorher umbauen: `graph` (1) und `rank` (1) verlieren den Bezug auf `MemoryLink`. `checkpoint` (4) und `ports` (3) verlieren `WorkflowState` mit der Auflösung der `compat/` in `checkpoint` (CMP-Auflösung) |
| `TlHfdParams` | `contextra-rank::temporal` | verschieben. Zeitlicher Zerfall gehört zu `rank` (REM-19 nennt `rank/src/temporal.rs`). Verbraucher prüfen |
| `PprAlgorithm`, `PprConfig` | `contextra-graph::ppr` | verschieben. Graph-Algorithmus. Verbraucher prüfen |
| `ConfigFingerprint` inklusive `with_threshold`, `threshold`, `temperature` | Besitzer nach Verbraucher | Verbraucher prüfen. Ohne Verbraucher entfernen (V8) |
| `AutoExtractionMode` | `contextra-ingest::config` | verschieben (E2). Verbraucher prüfen |

### 3.5 `types/saos.rs` (942 Zeilen, 45 Items)

| Item | Ziel | Aktion |
|---|---|---|
| `FusionStrategy`, `SignalFusionStrategies`, `uniform`, `strategy_for_signal`, `FusionWeights` inklusive `new`, `vector`, `text`, `graph`, `metadata` | `contextra-rank::fusion` | verschieben. Verbraucher: `rank` (7 und 4). Nur `rank` nutzt sie |
| `Signal`, `OnSignalFailure` | `contextra-ports::dto` | verschieben. V3: `rank` (16), `graph` (4), `vector`, `store`, `db`, `sys` (je 1). Den Sonderfall `sys` (1) prüft U-TYP-0 |
| `ContextChunk` inklusive `combined_text_owned`, `combined_token_count`, `has_context_prefix` | `contextra-ports::dto` | verschieben. V3: `ports` (4), `rank` (2), `text` (1), `testkit` (4) |
| `ContextWindow` | `contextra-ports::dto` | verschieben. `ports` (2), `testkit` (3) |
| `HybridQuery`, `QueryOptions`, `HybridQueryBuilder`, `QueryOptionsBuilder` inklusive aller `with_*`-Methoden | entfällt | entfernen. Ersetzt durch `SearchQuery` aus `contextra-ports::dto` (SPEC 15). Ein Abfragetyp, eine Definition (R4) |
| `ScoredEntry`, `QueryRewriteOutput` | Besitzer nach Verbraucher | Verbraucher prüfen. Ohne Verbraucher entfernen (V8) |
| `GraphTraversalStrategy` | entfällt | entfernen. 0 Verbraucher |

### 3.6 Übrige Module ohne Port-Bezug

| Datei | Items | Ziel | Aktion |
|---|---|---|---|
| `types/filter.rs` (427 Zeilen) | `FilterExpr`, `evaluate` | entfällt | entfernen. 0 Verbraucher. Filter sind Felder von `SearchQuery` (SPEC 15) |
| `types/importance.rs` (329 Zeilen) | `ImportanceScore`, `MemoryImportance`, `DecayFunction` inklusive Methoden | entfällt | entfernen nach REM-24. 0 Verbraucher |
| `erasure.rs` (617 Zeilen) | `ErasureRequest` (aus `EraseRequest` umbenannt) | `contextra-ports::dto` | verschieben. SPEC 15 nennt `ErasureRequest` |
| `erasure.rs` (Rest, 41 Items: `EraseTarget`, `Timing`, `EraseMode`, `RetentionCode`, `RetentionAttestation`, `EraseReceipt`, `LayerId`, `PendingReason`, `RefusalReason`, `Phase`, `ErasureStatus`, `KeyCoverage`, `GateVariant`, `MeasureClass`, `RiskCondition`, `ResidualRisk`, `Residue`, `Scanner`, `LayerWitness`, `ControlEvidence`, `Armed`, `Unarmed`) | entfällt, Ersatz `contextra-ports::erasure` | entfernen. Altmodell nach E-PRT-11. Ersatz: `SealState`, `RetireReason`, `ReadGate`, `ScrubTarget`, `ScrubPlan`, `ScrubProgress`, `IoBudget` |
| `error.rs` (1186 Zeilen, 21 Items) | `ContextraError` mit 16 Varianten, `Result` | `contextra-ports::error` | verschieben. Die Konstruktoren entfallen (U-TYP-6), siehe 3.1 |
| `error.rs` (Konstruktoren `limit_exceeded`, `model_load`, `snapshot_unsupported_for_signal`, `orphaned_vector_reference`, `capability_unsupported`, `invalid_input`, `wal_corruption`, `checksum_mismatch`, `kv_quantization`, `durability_config`, `pin_budget_exceeded`, `cross_device_link`, `wal_truncation_detected`, `is_occ_conflict`) | entfällt | entfernen. Jede Stelle wird auf eine der 16 Varianten abgebildet, siehe U-TYP-6. `is_occ_conflict` entfällt mit SSI (REM-12) |
| `error.rs` (`StepId`, `CacheDirective`) | Besitzer nach Verbraucher | Verbraucher prüfen. Ohne Verbraucher entfernen |
| `error_dto.rs` (614 Zeilen, `ContextraErrorDto`, `new`, `with_details`) | `contextra-mcp::error_payload` | verschieben. Nur das Antwortformat der MCP-Schnittstelle |
| `model_fingerprint.rs` (79 Zeilen) | `contextra-ports::embedding` | verschieben (siehe 3.1) |
| `retrieval_strategy.rs` (99 Zeilen) | `contextra-ports::dto` | verschieben (siehe 3.1) |
| `schema.rs` (129 Zeilen) | entfällt | entfernen (siehe 3.1) |
| `tombstone.rs` (159 Zeilen) | entfällt | entfernen (siehe 3.1) |
| `witness.rs` (98 Zeilen) | `RequestContext`, `SagaRecord`, `ScanMethod` | verschieben nach 3.1 |
| `tenant_scope.rs` (3 Zeilen), `refs.rs` (5 Zeilen), `ids.rs` (69 Zeilen), `types.rs` (37 Zeilen) | Re-Export-Dateien | entfernen (E-TYP-01) |

### 3.7 Konfiguration (SPEC 9 und 14)

- `ContextraConfig` (SPEC 9, Zeile 1219) gehört zur Fassade `contextra` (E2, Konfigurationsladen). Nicht in `ports`, nicht in `types`.
- Die Typen `EncryptionConfig`, `SecretString`, `GuardPolicy`, `DataClassPolicy`, `TenantPolicy`, `ErasureConfig`, `CustodyProfile`, `MixedPolicy` sind in SPEC 4.1 nicht enthalten (Suche ohne Treffer) und im Code nicht vorhanden. Sie entfallen. Die alte Fassung 2.1 führte sie als Kanon von `types`; das ist durch 4.1 überholt.

---

## 4. Entfernungen

| ID | Gegenstand | Grund | Ersatz |
|---|---|---|---|
| E-TYP-01 | `lib.rs` Re-Exporte (Zeilen 61–76) und Modulwurzeln `ids.rs`, `refs.rs`, `tenant_scope.rs`, `types.rs` | Re-Export-Schicht verhindert Besitz (F-08) | direkte Importe aus dem Besitzer |
| E-TYP-02 | Verzeichnis `crates/contextra-types/` mit 40 Dateien und 9.596 Zeilen | ADR-33 | — |
| E-TYP-03 | Workspace-Mitglied `contextra-types` und alle Abhängigkeitszeilen `contextra-types = …` in 266 Dateien | ADR-33 | Umstellung in U-TYP-5 |
| E-TYP-04 | Feature-Kopplung `workspace`, `types-compat`, `ports-compat` in `ports`, `mvcc`, `simd`, `checkpoint` | R3, R7 | — |
| E-TYP-05 | Agentengedächtnis: `LinkRelation`, `MemoryLink`, `MemoryType`, `WorkflowState`, `ImportanceScore`, `MemoryImportance`, `DecayFunction` | REM-24 | — |
| E-TYP-06 | Erasure-Altmodell in `erasure.rs` (außer `ErasureRequest`) | E-PRT-11 | `ports::erasure` |
| E-TYP-07 | Tombstone, Schema, `TenantScoped`, `DocKey`, `CollectionRef`, `TenantRef`, `OperatorRef`, `FilterExpr`, Konfiguration nach SPEC 14 | ADR-33, E-IDS-04 bis E-IDS-16 | Kennungen aus `ids`, `ContextraConfig` in `contextra` |
| E-TYP-08 | Konstruktoren in `error.rs` (14 Stück), `HybridQuery`, `QueryOptions` und Builder, `GraphTraversalStrategy` | siehe 3.4, 3.5 | `ContextraError`-Varianten, `SearchQuery` |
| E-TYP-09 | Kommentare und Dokumente, die `contextra-types` nennen | REM-19, Abnahme T-TYP-01 | — |

---

## 5. Umbauschritte

Jeder Schritt endet mit einem Build und einer Abnahme, bevor der nächste beginnt.

| ID | Schritt | Abhängig von | Abnahme |
|---|---|---|---|
| **U-TYP-0** | Bestand festschreiben. Für alle Items mit „Verbraucher prüfen“ und „Besitzer nach Verbraucher“ die Zahl der Fremdverwendungen ermitteln: `grep -rln "\bNAME\b" crates --include=*.rs \| grep -v contextra-types`. Ergebnis als Tabelle im PR. Ohne Verbraucher gilt V8 | keine | Tabelle vollständig |
| **U-TYP-1** | Additiv in `contextra-ports` neue Module einführen: `error.rs` (ContextraError mit den 16 Varianten aus `ports/PLAN.md`), `dto.rs` (ergänzen: `ScoredDocument`, `Signal`, `OnSignalFailure`, `ContextChunk`, `ContextWindow`, `TokenBudget`, `BudgetStrategy`, `Reservation`, `RetrievalStrategy`, `SearchQuery` nach SPEC 15), `embedding.rs` (`Embedding`, `ModelFingerprint`), `vector_index.rs` (`DistanceMetric`), `reranker.rs` (`RerankResult`), `context.rs` (`RequestContext`), `erasure.rs` (`ScanMethod`, `SagaRecord`, `ErasureRequest`). Alte Typen bleiben bestehen | U-IDS-1 bis U-IDS-4 | `cargo check -p contextra-ports`, Objektsicherheits-Compile-Test der Traits (SPEC 15) grün |
| **U-TYP-2** | `contextra-ids` übernimmt `SubjectRef`, `MAX_SEARCH_K`, `MAX_REF_LEN` (als `PrincipalRef::MAX_REF_LEN`), `TicketId` | U-IDS-1 bis U-IDS-4 | `grep -rn "MAX_SEARCH_K: usize"` liefert genau einen Treffer |
| **U-TYP-3** | `contextra-env::budget` mit `ResourceBudget` und `ResourceTracker` einführen. Die Zählerlogik bleibt unverändert | U-ENV-3 (Zeit-API) | `cargo test -p contextra-env` grün, T-ENV-Tests für Budget |
| **U-TYP-4** | Besitzer-Module in ihren Zielcrates anlegen: `contextra-vector::metric` (freie Funktionen `distance`, `distance_u8`), `contextra-vector::error` (`HnswDeletionError`), `contextra-graph::model` (`Entity`, `Edge`), `contextra-graph::ppr`, `contextra-rank::fusion`, `contextra-rank::temporal` (`TlHfdParams`), `contextra-ingest::config` (`AutoExtractionMode`), `contextra-mcp::error_payload` | Umbau der jeweiligen Zielcrates (eigene PLANs) | jeweiliger Crate-Test grün |
| **U-TYP-5** | Importe mechanisch umstellen: je Symbol ein Ersetzungsskript mit Zählung. Beispiel: `contextra_types::ContextraError` → `contextra_ports::error::ContextraError`. Pro Symbol vor und nach dem Lauf zählen, Abweichung bricht ab | U-TYP-1 bis U-TYP-4 | `grep -rln "contextra_types" crates` leer |
| **U-TYP-6** | Konstruktoren und Formen umbauen: 14 Konstruktoren in `error.rs` auf die 16 Varianten abbilden. `HybridQuery` und `QueryOptions` auf `SearchQuery`. Zeitfelder (`with_tx_validity`, `with_business_validity`, `with_validity`) von `TxId`/`Timestamp` auf `u64` | U-TYP-5 | `cargo test --workspace` für die betroffenen Crates grün |
| **U-TYP-7** | Crate entfernen: E-TYP-02, E-TYP-03, E-TYP-04. `Cargo.lock` aktualisieren. `forbidden_crate_names` in SPEC 1992 bleibt bestehen | U-TYP-5, U-TYP-6 | `cargo tree --workspace \| grep contextra-types` leer |

**Gesamtreihenfolge:** `ids` (U-IDS) → `env` (U-ENV-3, U-TYP-3) → `ports` (U-PRT-3, U-TYP-1) → Besitzer-Crates (U-TYP-4) → Verbraucher (U-TYP-5, U-TYP-6) → Entfernung (U-TYP-7). `contextra-mvcc`-Umbenennung wartet auf die Lösung von `store` (REM-27, siehe `contextra-mvcc/PLAN.md`).

---

## 6. Nachträge in anderen PLANs

Die Verteilung ist nur vollständig, wenn die Zielcrates die Typen auch aufnehmen. Folgende PLANs werden mit diesem Commit ergänzt:

- `crates/contextra-ports/PLAN.md`: Abschnitt „Nachtrag N-1“ mit den Modulen und Typen aus 3.1 bis 3.6, die `ports` übernimmt.
- `crates/contextra-env/PLAN.md`: Abschnitt „Nachtrag N-1“ mit `budget` (`ResourceBudget`, `ResourceTracker`). Das Ziel von U-ENV-4 (`SequenceLog`) heißt `contextra-version`, nicht `contextra-mvcc`.
- `crates/contextra-simd/PLAN.md`: Die Metrik-Enum `DistanceMetric` entfällt dort. Der Kern bietet nur Kernel-Funktionen.

---

## 7. Abnahme

| ID | Prüfung | Soll |
|---|---|---|
| T-TYP-01 | `grep -rln "contextra_types\|contextra-types" crates docs --include=*.rs --include=*.toml --include=*.md` | leer (außer Git-Historie) |
| T-TYP-02 | `cargo tree --workspace -e normal \| grep -c contextra-types` | 0 |
| T-TYP-03 | Inventar: jedes der 213 öffentlichen Items der Fassung ist in Abschnitt 3 mit Ziel und Aktion genannt | vollständig. U-TYP-0 gleicht die Zahl ab |
| T-TYP-04 | Jeder Typ hat nach Umbau genau eine Definition (R4). `grep -rn "pub struct DistanceMetric\|pub struct Embedding\|pub struct ScoredDocument"` | je ein Treffer |
| T-TYP-05 | `MAX_SEARCH_K` ist nur in `contextra-ids` definiert. Dieselbe Prüfung für `MAX_CHUNKS_PER_DOC` | je ein Treffer |
| T-TYP-06 | Keine Kante zwischen E1-Crates, die vorher über `types` lief (R2). Prüfung über `cargo tree -p <crate> -e normal` je E1-Crate gegen die Liste der E1-Crates in SPEC 3.2 | keine Treffer |

**Gate:** G-07 (F-08 aufgelöst).

---

## 8. Offene Entscheidungen

| ID | Frage | Vorschlag |
|---|---|---|
| O-TYP-1 | `ResourceTracker` in `contextra-env` statt in eigenem Crate | `env`. Beide Verbraucher (`store`, `graph`) haben `env` bereits, kein neuer Knoten im Graphen |
| O-TYP-2 | `TokenBudget` in `ports::dto` | Ja. `ports` nutzt den Typ in Signaturen. Verschieben nach `rank` würde einen Port-Kreis erzeugen |
| O-TYP-3 | `DistanceMetric` in `ports::vector_index` | Ja. `vector` ist der einzige Besitzer der Index-Logik, `simd` bekommt keine Metrik |
