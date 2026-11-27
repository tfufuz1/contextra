# Implementierungsspezifikation `contextra-graph` (Löschkette – Provenienz und Graph-Layer)

*Ring 0 · Konzept-WP 8 · Abhängig von: 02, 03, 10 · Index: `00-INDEX.md` (K-22, K-35, K-36, K-38)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

Der `GraphPurger` kennt nur Kanten mit `source_doc_id` und setzt `source_doc_id = Some(doc_id)` selbst (B-10). Ziel: Pflicht-Provenienz für abgeleitete Objekte, vollständige Bereinigung (Kanten, Entities, Hyperedges, Communities), `NeedsRecompute` bei Mehrquellen, Graph-Layer als `ErasureLayer`.

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `types`/`ports` optional, `parking_lot`, `tracing`, `arc-swap = "1.7"`. Keine Kante auf `store` |
| Provenienz | `provenance.rs`: `EdgeProvenance{edge_id, source_doc_ids: Vec<DocId>, created_at_tx}` (`:23,34`), `DocEdgeIndex{record, record_provenance, edges_for_doc, remove_doc}` (`:47-87`) |
| Quelle der Kante | `types.rs:170` `Edge.source_doc_id: Option<DocId>` (zweites Feld, K-22), `with_source_doc_id` (`:192`); `csr/types.rs:66,109,120` weitere `source_doc_id` |
| Purger | `purger.rs`: `GraphPurger::new(Arc<CsrGraph>)` (`:25`), `graph_purger()` (`:31`); `edge.source_doc_id = Some(doc_id)` (`:102`) |
| Persistenz | Schlüssel `__graph:entity:`, `__graph:edge:`, `__graph:community:` (`csr/types.rs:9-12`), **tenantweit**, im LSM (keine Datei-I/O im Crate außer Testdoubles, `graph_persist.rs` **[?]** prüfen) (K-38) |
| Graphstruktur | `CsrGraph` mit `ArcSwap`-Snapshots (`csr/inner`, `csr/visibility.rs`), Mutationen `csr/graph_write/{edge_mutation,entity,hyperedge,tombstone}.rs` |
| Witness | `WitnessLayer::CsrGraph` ✔ (Scanner: **[?]** `rg "impl LayerScanner" crates/contextra-graph`) |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `src/provenance.rs` | `DerivedFrom` (§4), `DocObjectIndex` ersetzt `DocEdgeIndex` rückwärtskompatibel (Alias) |
| `src/types.rs`, `src/csr/types.rs` | `derived_from: Vec<DocId>` an **allen** abgeleiteten Objekttypen (Edge, Entity, Hyperedge, Community) als Pflichtfeld (§4); `source_doc_id` bleibt als Legacy-Lesefeld |
| `src/erasure.rs` | **NEU**: `GraphErasure`, `GraphTxSource` |
| `src/csr/graph_write/*` | `purge_doc_batch` (§5.3) |
| `src/purger.rs` | `#[deprecated]`, nach WP-8.4 gelöscht |
| `src/csr/graph_persist.rs` | Schlüssel-Hilfsfunktionen für Provenienzindex (`__graph:prov:`) |
| `tests/graph_erasure_conformance.rs` | **NEU** |

## 4. Provenienz

```rust
/// Pflichtfeld an abgeleiteten Objekten. Leer nur bei ausdrücklicher Altbestands-Markierung.
pub struct Derivation { pub docs: Vec<DocId> /* sortiert, ohne Duplikate */, pub needs_recompute: bool, pub unprovenanced: bool }
```

- Ein `Vec<DocId>` statt `SmallVec` (**keine neue Abhängigkeit**, Index-Entscheid nennt SmallVec; hier bewusst abweichend: `smallvec` ist in `graph/Cargo.toml` nicht vorhanden ✔; Prüfauftrag O-G1, falls Allokationen im Heißpfad messbar sind).
- **Quelle ≠ Collection:** `DocId` ist tenantweit eindeutig nur zusammen mit `(tenant, collection)`; weil Graph-Schlüssel tenantweit sind (K-38), enthält `Derivation` zusätzlich `collection: CollectionId` je Quelle: `docs: Vec<(CollectionId, DocId)>`. **Normativ: `Vec<(CollectionId, DocId)>`.**
- Schreibpfade (`entity.rs`, `edge_mutation.rs`, `hyperedge.rs`, Extraktion in `engine/src/extraction/*`) setzen `derived_from` **beim Anlegen**; ein Anlegen ohne Quelle ist `Err(GraphError::MissingProvenance)`. Ausnahme: manuell angelegte (primäre) Objekte tragen die eigene Dokumentquelle, falls vorhanden, sonst `derived_from = []` und `primary = true` (werden nicht von Dokument-Erase berührt, nur von Collection-/Tenant-Erase).
- **Altbestand:** Objekte ohne Feld laden mit `unprovenanced = true`. Der Plan zählt sie in `ResolvedScope.unprovenanced` (Zahl pro Auftrag = Objekte des Mandanten ohne Provenienz); der Treiber meldet `ErasureException::Unprovenanced{objects}` und senkt die Garantie (I-6). Migration: Backfill-Worker (WP-8.5) leitet Provenienz aus `Edge.source_doc_id` ab; mehr ist nicht rekonstruierbar.
- **Provenienzindex (Rückwärtsrichtung):** `__graph:prov:{collection}:{doc_id}` → Liste `ObjectRef{kind u8, id u64}`. Wird in derselben Transaktion wie das Objekt geschrieben. Der Index selbst liegt im Schlüssel-Scope des Dokuments (Präfix `__graph:prov:{collection}:{doc}` exakt) ⇒ der Store entfernt ihn (03 §7). Der Graph-Layer liest ihn über `PrivilegedRead`.

## 5. Layer-Verhalten

### 5.1 `layer()` = `WitnessLayer::CsrGraph`

### 5.2 `on_gate(scope)` (RAM, < 10 ms)

`erased_docs` (je `(tenant, collection)`) per Copy-on-Write setzen. Traversierung (`ppr`, `path_rag`, `community`, `csr/graph_read/*`): Nachbar-/Kantenzugriff prüft `derived_from` gegen `erased_docs`; **ein** Objekt wird unsichtbar, wenn **alle** Quellen gesperrt sind (`docs ⊆ erased`); bei teilweiser Sperre bleibt es sichtbar, aber `needs_recompute`-Markierung wird erst in `purge` gesetzt. `ArcSwap`-Snapshot (`csr`) bleibt unverändert; die Filtermenge wird beim Lesen geprüft.

### 5.3 `purge(scope, cx)`

Einheit = **ein Objekt**. Arbeitsliste (sortiert, deterministisch) aus dem Provenienzindex:
1. Für jedes `(collection, doc)` der Dokumentsicht `cx.reader.get(__graph:prov:{collection}:{doc})` ⇒ `ObjectRef`-Liste. Collection-/Tenant-Scope: `scan_prefix(__graph:prov:{collection}:)` in 1 024er Seiten.
2. Cursor = `[ver u8][work_hash 8][done u64 LE]`.
3. Pro Objekt (Batch `cx.max_units`, Default 256), eine Transaktion pro Batch (`GraphTxSource::begin`, `StorageEngine::commit`):
   - `remaining = derived_from \ erased_docs`.
   - `remaining.is_empty()` ⇒ Objekt entfernen (`delete(__graph:edge:…)` bzw. `entity`/`community`; für Entities zuvor alle inzidenten Kanten prüfen: Kante ohne verbleibende Quelle fällt ohnehin, Kante mit anderer Quelle bleibt und der Entity-Endpunkt bleibt bestehen).
   - sonst `derived_from = remaining`, `needs_recompute = true` (nur bei Inhaltsfeldern, die aus Dokumentinhalt stammen: Summary, Beschreibung, Embedding des Objekts ⇒ geleert/`None`, bis neu berechnet), Objekt neu schreiben.
   - **Hyperedge:** Teilnehmer entfernen, deren einziger Beleg das Dokument ist; bleiben **< 2** Teilnehmer, wird die Hyperedge entfernt.
   - **Community/Summary:** Mitglieder-Objekte verändert ⇒ `needs_recompute = true`, `summary` auf `None`; Community ohne Mitglieder entfällt.
4. Tenantprüfung bei **jedem** Zugriff (`TxId::MAX`-Muster entfällt, B-10); Schlüssel müssen das Mandantenpräfix des Auftrags tragen sonst `ScopeViolation`.
5. Nach Commit: In-Memory-`CsrGraph` über die bestehenden Mutationspfade aktualisieren (`graph_write/*`, `tombstone.rs` nur für **logisches** Entfernen; hier zusätzlich `compact_now()`-Schritt aus `csr/inner/compaction.rs`, damit alte CSR-Arrays freigegeben werden). `ArcSwap`-Swap erst nach erfolgreicher Persistenz.
6. Collapse: Objektschlüssel (`__graph:*`) sind **nicht** im Collection-Schlüssel-Scope (K-38). Ältere Versionen (Altwerte mit Quell-Doc) müssen fallen ⇒ `cx.progress.register_collapse(&[(key, Seq(s))…])` (A-1), `s = last_seq_no()` nach Commit.
7. `PurgeOutcome{units_committed = persistierte Objekte}`. Teilfehler ⇒ `Err` (kein `Ok` mit Zähler).

### 5.4 Gate-Bruch Provenienz-Index

Der Provenienzindex wird in derselben Transaktion gepflegt wie das Objekt; beim Entfernen eines Objekts mit mehreren Quellen werden **alle** Rückwärtseinträge angepasst (auch für nicht betroffene Dokumente). Einträge fremder Dokumente liegen nicht im Scope ⇒ Writes sind erlaubt.

## 6. Persistenz-Inventar (K-38, O-G2)

Prüfauftrag vor WP-8.2: `rg "storage\.(put|delete)" crates/contextra-graph/src/csr/graph_persist.rs crates/contextra-graph/src/storage.rs`; Ergebnis dokumentiert, ob `GraphStorage` ausschließlich über `StorageEngine` schreibt. Erwartung: ja (✔ keine `std::fs` im Crate). Falls Snapshots/Dateien existieren, gilt §6 von 07 sinngemäß (`Vfs`).

## 7. Witness

Scanner `CsrGraph`: Strukturmessung (Kanten/Entities mit Quelle ∈ Scope, `remaining` = Anzahl) plus Byte-Nadeln (DocId LE/BE/Dezimal in serialisierten `__graph:*`-Werten **im RAM-Snapshot**; Dateien erledigt der Store-Scanner). `Unprovenanced > 0` ⇒ Ausnahme, nicht `remaining`.

## 8. Tests

| ID | Inhalt |
| --- | --- |
| TST-GRAPH-ERA-1…7 | Konformanz C1–C7 |
| TST-GRAPH-ERA-8 | Kante mit zwei Quellen: Erase von Quelle A ⇒ Kante bleibt, `derived_from = [B]`, `needs_recompute` für Inhaltsfelder |
| TST-GRAPH-ERA-9 | Kante mit Quelle A: Erase ⇒ Kante und Rückwärtseintrag weg; Nachbarn bit-identisch |
| TST-GRAPH-ERA-10 | Hyperedge: 3 Teilnehmer, einer fällt ⇒ bleibt; zwei fallen ⇒ entfernt |
| TST-GRAPH-ERA-11 | Traversierung (`ppr`, `path_rag`) nach `on_gate` liefert keine Objekte mit ausschließlich gesperrten Quellen |
| TST-GRAPH-ERA-12 | Altbestand ohne Provenienz: Plan meldet `unprovenanced > 0`, Seal weist `Unprovenanced` aus, Garantie ohne `purged`-Vollbild |
| TST-GRAPH-ERA-13 | Fremdmandant: Erase von Mandant 7 verändert `__graph:*` von Mandant 8 nicht (C3), Verletzungsversuch ⇒ `ScopeViolation` |
| TST-GRAPH-ERA-14 | `rg "TxId::MAX" crates/contextra-graph/src/erasure.rs` leer |
| TST-GRAPH-ERA-15 | Wiederaufnahme an jedem Batch-Cursor, Crash zwischen Commit und `record` ⇒ No-op bei Wiederholung |

## 9. Arbeitspakete

| WP | Inhalt |
| --- | --- |
| 8.1 | `derived_from`/`Derivation`, Pflicht beim Anlegen, Altbestand-Flag, Provenienzindex-Schlüssel |
| 8.2 | `GraphErasure` (`on_gate`, Filter in Traversierung), Konformanz-Skelett |
| 8.3 | `purge` (Batch, Hyperedges, Communities, Collapse) |
| 8.4 | Alt-Purger entfernen; Scanner-Nadeln |
| 8.5 | Backfill-Worker aus `source_doc_id` (Engine, `background-maintenance`) |

## 10. Definition of Done

Konformanz grün; Pflichtfeld erzwungen (Compile- oder Laufzeitfehler beim Anlegen ohne Quelle); `ErasureException::Unprovenanced` getestet; kein `let _ =` im Erase-Pfad.

## 11. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-G1 | `smallvec` statt `Vec` | Nur bei messbarem Heißpfad-Gewinn; sonst `Vec` |
| O-G2 | Persistenzpfad vollständig | siehe §6 |
| O-G3 | Besitzt `CsrGraph` bereits einen `LayerScanner`? | `rg "impl LayerScanner"`; sonst WP-8.4 anlegen |
| O-G4 | Semantik „primäre“ Entities (manuell angelegt) bei Dokument-Erase | unberührt; nur Collection-/Tenant-Scope entfernt sie |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `contextra-graph` enthält **keine** Erasure-Implementierung (kein `ErasureLayer`, kein Scanner, keine Referenz auf `Erasure*`). `EdgeProvenance`/`DocEdgeIndex` existieren (K-22), die Graph-Schlüssel sind tenantweit (K-38). Damit ist Datenort-Matrix **Zeile 13 offen (S1)**: ein `erase(Document)` lässt abgeleitete Entitäten, Kanten und Communities stehen.

**Abnahmekriterien für `GraphErasure`** (zusätzlich zu 08 §1–§10): (1) Layer-Konstante `LAYER_GRAPH` und Zeuge `Graph` (A-3-Stil); (2) Entfernen von Kanten/Entitäten **nur**, wenn ihre Provenienzmenge nach Abzug leer ist; Teilmengen werden um das Dokument verkleinert, Community-Zusammenfassungen, die gelöschten Text enthalten, werden verworfen und neu erzeugt oder als `Stale` markiert und gesperrt; (3) CSR-Neuaufbau über `StoreIo`, Temp/`rename`; (4) C8-Ergebnisfilter für jede Graph-Abfrage; (5) `unprovenanced`-Daten (Altbestand ohne Provenienz) werden im `SealReceipt` als Ausnahme ausgewiesen und senken die Garantie (I-6); (6) Canary `TST-GRAPH-CANARY-1`.
