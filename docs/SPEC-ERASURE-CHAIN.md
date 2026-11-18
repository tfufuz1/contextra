# SPEC: Contextra Erasure Chain — Vollständige Löschkette (SOLL-Zustand)

> **Dokument-Typ:** Implementierungsspezifikation  
> **Version:** 1.0  
> **Status:** FINAL — Referenz für alle Implementierer  
> **Scope:** `contextra-erasure` (Ring 3), `contextra-ports` (Ring 2), alle Layer-Implementierer in Ring 4+

---

## 1. Ziele & Prinzipien

| ID | Prinzip |
|----|---------|
| P-1 | **Vollständigkeit:** Jede Löschung läuft durch alle 6 Schritte bis `Completed` oder landet terminal in `Refused`/`Pending` mit Recovery-Pfad. |
| P-2 | **Idempotenz (PO-05):** Jeder Schritt — insbesondere `purge` und `witness` — ist beliebig oft wiederholbar ohne Datenverlust oder Doppelfreigabe. |
| P-3 | **Cancel-Safety (ER-24):** Der Aufrufer darf sein Future jederzeit droppen. Die Saga läuft in einer unabhängigen Tokio-Task bis mindestens `Gated` durch. |
| P-4 | **Kein PII in Logs/Observability (D-118):** Alle Observer-Events tragen nur `ErasureId`, `TargetKind` und `ObservedPhase`. Niemals Schlüssel-/Inhaltsbytes. |
| P-5 | **fsync-Disziplin (D-58):** Genau 4 fsync-Ankerpunkte bis `Gated`. Jeder persistente Zustandsübergang ist durch einen Journal-Append gedeckt. |
| P-6 | **Ring-Isolation (AR-3):** `contextra-erasure` darf nur `contextra-types`, `contextra-ports` und `contextra-crypto` importieren. Keine Abhängigkeit auf konkrete Speicher-Crates. |
| P-7 | **Tenant-Isolation:** Kein Schritt einer Saga berührt Daten außerhalb des im `EraseTarget` spezifizierten Schlüsselbereichs. |

---

## 2. Crate-Topologie & Verantwortlichkeiten

```
Ring 1 — contextra-ids          (IDs: ErasureId, FileId, GateId, Seq, Timestamp)
Ring 2 — contextra-types        (Domänenmodell: EraseTarget, SagaRecord, RetentionAttestation, …)
Ring 2 — contextra-ports        (Port-Traits: ErasureLayer, ReadGate, CommitBarrier, PurgeBarrier, …)
Ring 3 — contextra-erasure      (Löschkern: ErasureCore, EraseSaga, Journal, Gate, Vault, Measure, Ledger, Report)
Ring 4+ — Layer-Implementierer  (contextra-store, contextra-vector, contextra-text, …) implementieren ErasureLayer
```

> [!IMPORTANT]
> **contextra-erasure-spi (Decision D-E1, zukünftig):** Alle aktuell in `contextra-ports::erasure_legacy` als `#[deprecated]` markierten Traits sollen in ein neues Crate `contextra-erasure-spi` migrieren. Dies ist ein **geplanter Folgeschritt** und blockiert nicht den aktuellen SOLL-Zustand. Bis dahin: alle Traits in `contextra-ports::{erasure, barrier, gate, keys, ledger, observer, report_signer, scanner}` sind die kanonischen Quellen.

---

## 3. Domänenmodell: Kanonische Typen

Alle nachfolgenden Typen leben in `contextra-erasure::types_local` (intern) bzw. werden aus `contextra-ports` und `contextra-types` re-exportiert. Es gibt **keine zweiten Definitionen**.

### 3.1 Identifikatoren

```rust
// Ring 1 / contextra-types
pub struct ErasureId(pub [u8; 16]);   // Eindeutige Saga-ID
pub struct FileId(pub u64);            // Datei im Keyring
pub struct GateId(pub [u8; 16]);       // Gate-Log-Eintrag
pub struct Seq(pub u64);               // Monoton steigende Schreibsequenz
pub struct Timestamp(pub u64);         // Unix-Zeit in ms (Clock-Abstraktion)
pub struct LayerId(pub [u8; 16]);      // Layer-Identifikator (statisch, pro Implementierung)
```

### 3.2 Lösch-Ziel

```rust
pub enum EraseTarget {
    // Halboffenes Intervall [start, end) — für Tenant- und Collection-Löschungen
    KeyRange { start: Vec<u8>, end: Vec<u8> },
    // Exakte Schlüsselmenge — für Einzel-Dokument-Löschungen
    ExactKeys(Vec<Vec<u8>>),
}
```

> [!NOTE]
> `EraseTarget` existiert in zwei Parallelwelten: `contextra-ports::erasure::EraseTarget` (Ports-seitig, für `ErasureLayer`) und `contextra-erasure::types_local::EraseTarget` (Kern-seitig, für `ErasureCore`/`SagaSeams`). **Beide sind strukturell identisch.** Der Kern übersetzt bei Bedarf via `Into`-Impl (kein Clone-Overhead, kein semantischer Unterschied). Die Implementiererspezifikation in §6 klärt welche Variante Layer-Implementierer verwenden.

### 3.3 Saga-Zustände

```
Requested ──► Gated ──► KeyDestroyed ──► Purging ──► Verifying ──► Completed
    │             │            │              │              │
    │             └────────────┴──────────────┴──────────────┴──► Pending(reason) [wiederholbar]
    ├──► Restricted ──► Lifted  (Art. 18 DSGVO, D-116)
    │         └──► Gated       (Restriction wird durch Erase aufgehoben, D-15/D-119)
    └──► Refused               (terminal, Retention-Pflicht aktiv)
```

**Invarianten:**
- `Completed`, `Refused`, `Lifted` sind terminale Zustände (kein weiterer Übergang erlaubt außer Idempotenz).
- `Pending` ist kein terminaler Zustand — Recovery durch Hintergrundläufer (`run_purge_window`).
- Gleicher → Gleicher Übergang ist immer erlaubt (Idempotenz, ER-18).
- `Gated → Purging` ist direkt erlaubt (D-117: Superseded-Version ohne separaten KeyDestroy-Schritt).

### 3.4 Rückhalte-Attestierung

```rust
pub struct RetentionAttestation {
    pub code: RetentionCode,
    pub operator_ref: OperatorRef,  // Betreiber-Kennung, [REDACTED] in Logs
}

pub enum RetentionCode {
    NoObligation,                      // → Erase erlaubt
    ObligationExpired,                 // → Erase erlaubt
    OperatorOverride,                  // → Erase erlaubt
    ObligationActive,                  // → Erase verweigert, Restrict erlaubt
    LegalClaimsPending,                // → Erase verweigert, Restrict erlaubt
    DataSubjectRequestedRestriction,   // → Erase verweigert, Restrict erzwungen
}
```

### 3.5 Schicht-Zeuge (LayerWitness)

```rust
// Kanonische Definition: contextra-ports::scanner::LayerWitness
// (INV-WITNESS-MINT-1: Nur eine Definition — keine zweite in erasure.rs)
pub struct LayerWitness {
    pub layer_id: LayerId,
    pub purged_count: u64,
    pub bytes_reclaimed: u64,
    pub residue_matches: u64,  // Muss 0 sein für Completed
    pub scan_class: MeasureClass,
}
```

---

## 4. Die 6-Schritte-Saga (SPEC §11.2)

Alle 6 Schritte werden von `ErasureCore::execute_erase_flow` orchestriert. Die Implementierer der Nähte (Seams) implementieren die konkreten Aktionen.

### Schritt 1 — Validierung, Resolve, Intent-Journal (fsync #1)

**Akteur:** `ErasureCore`

1. **Retention-Matrix prüfen** (D-124 Abs. 6): `evaluate_retention(mode, attestation)` → `ProceedErase | ProceedRestrict | Refuse(reason)`.
2. **Open-Key-Duplikatprüfung** (D-119): HMAC-SHA256(`k_idx ‖ target_bytes ‖ mode_byte`) → wenn Saga bereits aktiv, Receipt zurückgeben (idempotent).
3. **Resolve je Schicht**: Für jeden `Arc<dyn ErasureLayer>` in `deps.layers` → `layer.resolve(&target).await` → `LayerScope`. Fehler → `Pending(LayerFailed)`.
4. **Journal Intent** (fsync #1): `journal.append_intent(erasure_id, &target, &attestation).await`.

**Fehlerpfad:** `LayerFailed` → Zustand `Pending`, Receipt mit `Phase::Intent`, Saga in `active_sagas` eintragen, Return.

---

### Schritt 2 — Gate setzen (fsync #2)

**Akteur:** `ErasureCore` unter `CommitBarrier::exclusive`

1. `deps.commit_barrier.exclusive().await` — RAII-Permit hält Lock bis Drop.
2. `deps.gate.add_gate(erasure_id, &target).await` → `gated_seq: u64`.
3. `saga.transition_to(SagaState::Gated)`.
4. **Journal Gated** (fsync #2): `journal.append_state(erasure_id, Gated, None).await`.
5. Permit dropen → andere Writers können committen.

**Kritische Invariante:** Zwischen `add_gate` und dem Journal-Append darf **kein** anderer Writer einen Commit durch führen. Die `CommitBarrier` stellt dies sicher.

---

### Schritt 3 — on_gate + Schlüsselvernichtung (fsync #3)

**Akteur:** `ErasureCore`

1. Für jeden Layer: `layer.on_gate(&scope).await` — z.B. Vektor-Surrogate markieren. Fehler → `Pending(LayerFailed)`.
2. `deps.vault.destroy_highest_key(&target).await` — vernichtet den höchsten Schlüsselslot (T/C/S/D-Hierarchie). **fsync #3 intern im Vault.**
3. `deps.vault.verify_key_absent(&target).await` — verifiziert Null-Zustand. Fehler → `Pending(Io)`.

**Semantik `on_gate`:** Bereitet RAM-Zustand vor. Persistente Markierung (z.B. Tombstone im Vektor-Index) erfolgt im Gruppen-Commit. **Darf keine I/O-Fehler ignorieren.**

---

### Schritt 4 — Journal KeyDestroyed + Observer-Benachrichtigung (fsync #4)

**Akteur:** `ErasureCore`

1. `saga.transition_to(SagaState::KeyDestroyed)`.
2. **Journal KeyDestroyed** (fsync #4): `journal.append_state(erasure_id, KeyDestroyed, None).await`.
3. Observer-Event `ObservedPhase::Gated` (Benennungskonvention: "Gated" bedeutet "Daten sind unlesbar", unabhängig von physischem Purge).
4. Saga in `active_sagas` eintragen, Open-Key registrieren.
5. **Return `EraseReceipt { status: Gated, … }`** — der Aufrufer erhält sofort Kontrolle zurück.

> [!IMPORTANT]
> Ab hier laufen Schritt 5 und 6 entweder synchron (wenn `timing == Timing::Now`) oder asynchron via `run_purge_window`. Der Aufrufer ist bereits aus dem kritischen Pfad entlassen. Cancel-Safety ist erfüllt.

---

### Schritt 5 — Purge-Fenster (Reihenfolge a–g)

**Akteur:** `ErasureCore::execute_purge_and_verify` unter `PurgeBarrier::purge`

**Vorbedingungen:**
- `PurgeBarrier::purge().await` — exklusiv gegen Kompaktionen (ER-21).
- `check_disk_reserve(min_free_bytes, 0)` — Plattenreserve prüfen (ER-22). Fehler → `Pending(DiskReserve)`.

**Reihenfolge der Sub-Schritte (a–g) nach `PurgePriority`:**

| Prio | Sub-Schritt | Akteur | Beschreibung |
|------|-------------|--------|--------------|
| a | Snapshot-Widerruf | `SnapshotRevoker` | `revoke_covering(&target, gated_seq)` — invalidiert alle Snapshots, die Daten ≤ `gated_seq` enthalten |
| b | Text-Segment-Rewrite | `ErasureLayer` (TextLayer) | Physisches Überschreiben von Textsegmenten |
| c | Memtable-Drain / WAL | `ErasureLayer` (StoreLayer) | WAL-Einträge flushen und Memtable leeren |
| d | SSTable-Rewrite | `ErasureLayer` (StoreLayer) | Kompaktions-Rewrite der SSTables ohne gelöschte Keys |
| e | Vektor-Reparatur | `ErasureLayer` (VectorLayer) | HNSW-Graph-Reparatur, surrogat-markierte Vektoren entfernen |
| f | D-Slots / Slot-Bereinigung | `ErasureLayer` (Store/Crypto) | Verbleibende Datei-Schlüsselslots nullen |
| g | Cache-Invalidierung | Alle Layer | In-Memory-Caches leeren |

**Innerhalb jedes Layers:**
1. `layer.purge(&scope, &purge_cx).await` → `PurgeOutcome { purged_count, bytes_reclaimed }`.
2. Fehler → `Pending(LayerFailed)`, Saga aktualisieren, Return. Der Hintergrundläufer wird den Schritt wiederholen.

**Journal:** `append_state(erasure_id, Purging, None)` **vor** dem ersten `layer.purge`-Aufruf.

---

### Schritt 6 — Messung, Bericht, Ledger, Abschluss

**Akteur:** `ErasureCore::execute_purge_and_verify` (Fortsetzung)

1. `saga.transition_to(SagaState::Verifying)`.
2. `journal.append_state(erasure_id, Verifying, None)`.
3. **Witness je Schicht:** `deps.measure.witness_layer(layer_id, &target).await` → `LayerWitness`. Bei `ErasureError::Residue(_)` oder `ErasureError::Corrupt` → `Pending(WitnessMissing)`.
4. **Bericht signieren:** `deps.report.sign_report(erasure_id, &target, &attestation, &witnesses).await` → `Vec<u8>` (Ed25519-signierter JSON-Bericht).
5. **Ledger schreiben:** `deps.ledger.record_completed(erasure_id, &target, gated_seq, completed_at).await` — außerhalb des Datenverzeichnisses (ER-30).
6. `saga.transition_to(SagaState::Completed)`.
7. `journal.append_state(erasure_id, Completed, None)`.
8. `deps.gate.compact_gate(erasure_id).await` — Gate-Eintrag bereinigen.
9. Observer-Event `ObservedPhase::Completed`.
10. `active_sagas` und `open_keys` aufräumen. Purge-Permit droppen.

---

## 5. Seams (Nähte) — Vollständige Schnittstellenspezifikation

Alle Seams sind asynchrone Traits mit `Send + Sync`. Der Kern (`ErasureCore`) kennt nur diese Abstraktionen.

### 5.1 `SagaJournal`

```rust
pub trait SagaJournal: Send + Sync {
    // fsync #1: Initialer Intent-Eintrag mit Ziel und Attestierung
    fn append_intent<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
        attestation: &'a RetentionAttestation,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    // fsync #2–#6: Zustandsübergang anhängen
    fn append_state<'a>(
        &'a self,
        erasure_id: ErasureId,
        state: SagaState,
        reason: Option<&'a PendingReason>,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    // Recovery: Alle noch nicht abgeschlossenen Sagas liefern
    fn recover_open_sagas<'a>(
        &'a self,
    ) -> BoxFuture<'a, Result<Vec<EraseSaga>, ErasureError>>;
}
```

**Implementierungs-Anforderungen:**
- Jeder Append muss dauerhaft sein (fsync oder O_DSYNC) **bevor** `Ok(())` zurückgegeben wird.
- Format: Append-only Log, keine in-place Mutations. Implementierung: `DurableJournal` (via `contextra-durable-fs` / `Vfs`-Abstraktion).
- `recover_open_sagas`: Reduziert den Log auf den letzten Zustand je `ErasureId`, filtert `Completed` und `Refused` heraus.

---

### 5.2 `SagaGate`

```rust
pub trait SagaGate: Send + Sync {
    // Lesesperre eintragen + Gate-Log persistieren (fsync #2)
    // Rückgabe: gated_seq (monoton steigende Schreibsequenz des Stores)
    fn add_gate<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<u64, ErasureError>>;

    // Gate-Eintrag nach Completed bereinigen
    fn compact_gate<'a>(
        &'a self,
        erasure_id: ErasureId,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    // Synchrone Gate-Abfrage: Open | Erased | Restricted
    fn decision_for_target<'a>(
        &'a self,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<GateDecision, ErasureError>>;
}
```

**Implementierungs-Anforderungen:**
- `add_gate` muss atomar sein: entweder vollständig persistent oder gar nicht (kein partieller Commit).
- `gated_seq` ist die aktuelle WAL-Schreibsequenz — alle Einträge mit `seq ≤ gated_seq` sind von diesem Gate abgedeckt.
- Der synchrone `ReadGate`-Trait (für Hot-Path-Lesepfade) wird **separat** implementiert und teilt intern den Zustand mit `SagaGate`.

---

### 5.3 `SagaVault`

```rust
pub trait SagaVault: Send + Sync {
    // Höchsten Schlüsselslot vernichten (fsync #3 intern)
    // Schlüsselhierarchie: Tenant(T) → Collection(C) → Segment(S) → Document(D)
    fn destroy_highest_key<'a>(
        &'a self,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    // Lesende Verifikation: Ist der Slot wirklich abwesend/nulled?
    fn verify_key_absent<'a>(
        &'a self,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<bool, ErasureError>>;
}
```

**Implementierungs-Anforderungen:**
- `destroy_highest_key`: Slot wird mit Zufallsbytes überschrieben, dann genullt, dann fsync. Danach MAC-Invalidierung des Slots.
- `verify_key_absent`: Rein lesend. Prüft Slot-Inhalt auf Null-Pattern via constant-time compare (`subtle::ct_eq`). Gibt `Ok(true)` nur wenn Slot definitiv absent/null.
- **Zeroize-on-Drop** für alle temporären Schlüsselkopien (`zeroize` Crate, `Zeroize`-Derive).

---

### 5.4 `SagaMeasure`

```rust
pub trait SagaMeasure: Send + Sync {
    // Positivkontrolle + Residuen-Scan für eine Schicht
    fn witness_layer<'a>(
        &'a self,
        layer_id: LayerId,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<LayerWitness, ErasureError>>;
}
```

**Implementierungs-Anforderungen:**
- Rein lesend (PO-05). Darf niemals persistenten Zustand verändern.
- Für Schicht mit `KeyCoverage::Full`: Scan nach logischen Einträgen im Zielbereich. `residue_matches > 0` → `Err(ErasureError::Residue(n))`.
- Für Schicht mit `KeyCoverage::None`: `residue_matches = 0` nach physischem Purge akzeptabel.
- MeasureClass-Hierarchie: `Mk1` (logisch) → `Mk2` (Byte-Scan) → `Mk3` (physisch/Medium).

---

### 5.5 `SagaReport`

```rust
pub trait SagaReport: Send + Sync {
    // Kanonischen, signierten Löschbericht erstellen
    fn sign_report<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
        attestation: &'a RetentionAttestation,
        witnesses: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<Vec<u8>, ErasureError>>;
}
```

**Implementierungs-Anforderungen:**
- JSON-Kanonisierung (kein PII, kein Klartextinhalt).
- Ed25519-Signatur via `ed25519-dalek` (Feature `signing`).
- Bericht-Inhalt: `erasure_id`, `target_kind` (kein Klartextinhalt), `gated_seq`, `completed_at`, `witnesses[]` mit `LayerWitness`-Daten, `operator_ref` (opak), `attestation.code`.

---

### 5.6 `SagaLedger`

```rust
pub trait SagaLedger: Send + Sync {
    // Abschlusseintrag außerhalb des Datenverzeichnisses schreiben (ER-30)
    fn record_completed<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
        gated_seq: u64,
        completed_at: u64,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;
}
```

**Implementierungs-Anforderungen:**
- Physisch **außerhalb** des Haupt-Datenverzeichnisses (separates Laufwerk oder separater Pfad).
- Hash-Verkettung: `LedgerEntry::compute_chained` mit `prev_hash` aus letztem Eintrag.
- HMAC-MAC über jeden Eintrag für Integritätsschutz.
- Prune-Funktion: Einträge älter als konfigurierbarer Retention-Zeitraum löschen (nicht erzwungen im Core).

---

### 5.7 `SnapshotRevoker`

```rust
pub trait SnapshotRevoker: Send + Sync {
    // Snapshots des Mandanten für betroffenen Bereich bis gated_seq widerrufen
    fn revoke_covering<'a>(
        &'a self,
        target: &'a EraseTarget,
        gated_seq: u64,
    ) -> BoxFuture<'a, Result<u64, ErasureError>>;
    // Rückgabe: Anzahl widerrufener Snapshots
}
```

**Implementierungs-Anforderungen:**
- Idempotent: Bereits widerrufene Snapshots ergeben kein Fehler, nur `count=0`.
- Widerruf bedeutet: Snapshot-Metadaten als ungültig markieren, Wiederherstellung aus diesem Snapshot muss fehlschlagen.
- Darf keine Snapshots anderer Tenants berühren.

---

## 6. ErasureLayer — Layer-Implementierer-Spezifikation

Jede Speicher- oder Indexschicht, die an der Löschkette teilnimmt, **muss** `contextra-ports::erasure::ErasureLayer` implementieren.

```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;           // Statisch, unveränderlich
    fn key_coverage(&self) -> KeyCoverage; // Full | Unlinked | None

    fn resolve(&self, t: &EraseTarget)
        -> BoxFuture<'_, Result<LayerScope, String>>;

    fn on_gate(&self, s: &LayerScope)
        -> BoxFuture<'_, Result<(), String>>;

    fn purge(&self, s: &LayerScope, cx: &PurgeCx)
        -> BoxFuture<'_, Result<PurgeOutcome, String>>;

    fn witness(&self, s: &LayerScope, sc: &Scanner<Armed>)
        -> BoxFuture<'_, Result<LayerWitness, String>>;
}
```

### 6.1 Layer-Rollen und Prioritäten

| Layer | KeyCoverage | PurgePriority | Zuständigkeit |
|-------|-------------|---------------|---------------|
| `StoreLayer` | `Full` | `WAL_DRAIN` (c), `SSTABLE_REWRITE` (d) | WAL-Flush, SSTable-Kompaktions-Rewrite |
| `TextLayer` | `Full` | `TEXT_REWRITE` (b) | Inverted-Index-Segment-Rewrite |
| `VectorLayer` | `Full` | `VECTOR_REPAIR` (e) | HNSW-Graph-Reparatur, Surrogate entfernen |
| `CryptoLayer` | `Full` | `SLOT_CLEANUP` (f) | D-Slots und residuale Datei-Schlüssel nullen |
| `CacheLayer` | `None` | `CACHE_FLUSH` (g) | In-Memory-Caches leeren |

### 6.2 `resolve` — Vorab-Scope-Ermittlung

- Darf **keine** Daten verändern.
- Liefert: Welche SSTable-Dateien, Segmente, WAL-Positionen vom Ziel betroffen sind.
- `LayerScope::affected_units`: Datei-IDs oder Segment-Nummern.
- Fehler: `Err(String)` mit ASCII-Fehlerbeschreibung (kein PII).

### 6.3 `on_gate` — Sofortmaßnahmen vor Purge

- RAM-Zustand vorbereiten (z.B. Vektor-Surrogate in-memory markieren).
- Darf minimale persistente Markierungen schreiben (z.B. Tombstone-Bit in WAL-Header), aber **kein** physisches Löschen.
- Idempotent.

### 6.4 `purge` — Physisches Löschen

- **Idempotent (PO-05):** Wiederholter Aufruf mit gleichem `LayerScope` ist sicher.
- Darf keine Daten außerhalb von `LayerScope::affected_units` berühren.
- `PurgeCx::deadline_ms`: Wenn Deadline überschritten → `Err("deadline exceeded")` (der Kern setzt in `Pending`).
- Strategie je Layer:
  - **Store:** SSTable-Rewrite mit Skip der gelöschten Key-Ranges, dann atomares Rename.
  - **Text:** Segment-Rewrite ohne gelöschte DocIds, dann Segment-Rotation.
  - **Vector:** HNSW-Knoten-Entfernung, Graph-Reparatur (Nachbarverbindungen aktualisieren), Datei-Rewrite.

### 6.5 `witness` — Residuen-Verifikation (read-only)

- **Ausschließlich lesend (PO-05).** Darf niemals Daten verändern.
- Erhält `&Scanner<Armed>` als Typzustand-Beweis, dass Positivkontrolle stattgefunden hat.
- Prüft: Sind noch Einträge im `LayerScope` mit Ziel-Keys vorhanden?
- `LayerWitness::residue_matches == 0` → Verifikation bestanden.
- `residue_matches > 0` → Implementierer gibt `Ok(witness)` zurück mit `residue_matches > 0`; der Kern interpretiert dies als `Err(Residue)`.

---

## 7. ReadGate — Synchroner Lesepfad

Der `ReadGate`-Trait ist der **synchrone Hot-Path** in allen Speicher- und Index-Lesepfaden.

```rust
pub trait ReadGate: Send + Sync {
    fn check(&self, k: &KeyBuf) -> GateDecision;
    fn check_bound(&self, b: &KeyBound) -> GateDecision;
    fn variant_for_range(&self, r: &KeyRange) -> RangeGate;
    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>;
    fn gate_epoch(&self) -> u64;    // Standard: 0
    fn covers(&self, k: &KeyBuf) -> bool;
    fn check_raw(&self, raw: &[u8; 78]) -> GateDecision;  // Standard: Open
    fn covers_raw(&self, raw: &[u8; 78]) -> bool;
}
```

**Implementierungspflichten für alle Speicher-Lesepfade:**
1. **Vor jedem Key-Read:** `gate.check(k)` aufrufen.
2. `Erased` → Key darf nicht zurückgegeben werden (leere Antwort / KeyNotFound).
3. `Restricted` → Key darf nicht zurückgegeben werden ohne vorherigen Audit-Eintrag (`RestrictedReadAuditRequired`).
4. **Vor jeder Kompaktions-Range:** `gate.variant_for_range(r)` aufrufen.
5. `ContainsErased` → Range nicht in Kompaktionsergebnis aufnehmen (Daten werden durch Purge-Phase entfernt).
6. `RestrictedNeverDiscard` → Range darf nicht kompaktiert werden, bis Restriction aufgehoben.

---

## 8. Barrieren — Concurrency-Koordination

### 8.1 `CommitBarrier` (D-57, D-78)

Schützt den Gate-Gruppen-Commit gegen parallele Writer.

```
Writer 1: commit_barrier.shared().await → [schreibt Daten]
Writer 2: commit_barrier.shared().await → [schreibt Daten]
Saga:     commit_barrier.exclusive().await → [setzt Gate] → drop permit
```

- `shared()`: Mehrere Writer können gleichzeitig halten.
- `exclusive()`: Exklusiv — wartet auf alle `shared`-Permits, blockiert neue.
- Implementierung: `tokio::sync::RwLock`-basiert oder äquivalent (kein `std::sync::Mutex` über `.await`).

### 8.2 `PurgeBarrier` (ER-21)

Schützt den Purge-Lauf gegen parallele Kompaktionen (und vice versa).

```
Kompaktions-Job: purge_barrier.compaction().await → [kompaktiert]
Saga-Purge:      purge_barrier.purge().await → [exklusiver Purge] → drop permit
```

- `compaction()`: Shared — mehrere Kompaktionen parallel erlaubt.
- `purge()`: Exklusiv — wartet auf alle Kompaktions-Permits.
- `purge_waiting()`: Synchrone Prüfung ob ein Purge wartet (Kompaktionen sollen kurz yielden).

---

## 9. Hintergrundläufer (`run_purge_window`)

```rust
impl ErasureCore {
    pub async fn run_purge_window(&self) -> Result<WindowOutcome, ErasureError>;
}
```

Verarbeitet alle Sagas im Zustand `KeyDestroyed` oder `Purging`:
1. `active_sagas` lesen → Kandidaten filtern.
2. Für jeden Kandidaten: `execute_purge_and_verify` aufrufen.
3. `WindowOutcome` akkumulieren.

**Betriebsmodell:** Wird von einem externen Scheduler periodisch aufgerufen (z.B. alle 60 Sekunden via `ErasureConfig::batch_window`). **Nicht** in einem internen `tokio::spawn`-Loop — der Host-Prozess steuert die Scheduling-Frequenz.

---

## 10. Restrict-Modus (Art. 18 DSGVO, D-116)

Separater Pfad für `EraseMode::Restrict`:

1. `saga.transition_to(SagaState::Restricted)`.
2. `journal.append_intent(…)` + `gate.add_gate(…)` → Gate setzt `GateDecision::Restricted`.
3. `journal.append_state(erasure_id, Restricted, None)`.
4. Observer `ObservedPhase::Restricted`.
5. **Return `EraseReceipt { status: Restricted, … }`** — kein physischer Purge.

**Aufhebung (`Lifted`):**
- Externer Aufruf setzt Saga von `Restricted` → `Lifted`.
- `gate.compact_gate(erasure_id)` entfernt Restriction.
- Saga ist terminal (`Lifted`).

**Restriction-to-Erase (D-15, D-119):**
- Übergang `Restricted → Gated` erlaubt.
- Vollständige Löschsaga startet neu ab Schritt 2 (Gate re-setzen als `Erased`).

---

## 11. Fehlertypen-Hierarchie

```rust
pub enum ErasureError {
    // I/O & Storage
    Io(std::io::Error),          // Handle danach ungültig (D-55)
    Storage(String),
    DiskReserve,                 // Plattenreserve unterschritten

    // Crypto & Vault
    Crypto(String),              // AEAD / MAC-Fehler
    Corrupt,                     // Slot-MAC ungültig
    Capacity,                    // Vault-Lastfaktor > 0.7
    SlotMissing,                 // Erwarteter Slot nicht vorhanden
    KeyRevoked(String),

    // Saga-Kontrolle
    InvalidTransition { from, to }, // Unerlaubter Zustandsübergang
    Refused,                        // Retention-Pflicht aktiv
    Residue(u64),                   // Scan-Residuen gefunden (count)

    // Spezialfälle
    ReplacementMissing,          // Replace-Modus: Ersatz fehlt (D-115)
    UpdateMissing,               // Supersede-Modus: Update fehlt (D-117)
    SameIdentity,                // Replace: Alt == Neu (D-115)
    RestrictionViolated(String), // Zugriff auf restricted Data
    RestrictedReadAuditRequired, // Audit-Eintrag fehlt (D-116, I-26)

    // Purge-Phase
    PurgePhaseTransition { from, to },  // Illegaler Phasenübergang
    PurgePhaseError { phase, reason },  // Fehler in spezifischer Purge-Phase

    // Infrastruktur
    LimitExceeded(String),
    Json(String),
    Ledger(String),
    Internal(String),            // Assertions / unerwartete Zustände
    NotImplemented(String),      // Nur in Entwicklungsphasen — nie in Produktion
}
```

> [!CAUTION]
> `ErasureError::Io` invalidiert den Handle. Der Rückgabepfad ist immer `Err`, der Layer-Zustand ist poisoned. Recovery: Saga in `Pending(Io)`, Layer beim nächsten Purge-Versuch neu initialisieren.

---

## 12. Invarianten-Checkliste für Implementierer

Jeder Layer-Implementierer und jede Seam-Implementierung muss folgende Invarianten einhalten:

| ID | Invariante | Betrifft |
|----|------------|----------|
| INV-1 | `purge` ist idempotent: zweiter Aufruf mit gleichem Scope ≡ no-op ohne Fehler | Alle Layer |
| INV-2 | `witness` ändert niemals persistenten Zustand | Alle Layer |
| INV-3 | `LayerWitness` wird nur in `contextra-ports::scanner` definiert (INV-WITNESS-MINT-1) | ports-Crate |
| INV-4 | `LayerId` eines Layers ist statisch und unveränderlich nach Initialisierung | Alle Layer |
| INV-5 | `CommitBarrier::exclusive` wird nur innerhalb von Schritt 2 gehalten — nie über mehrere Saga-Schritte | ErasureCore |
| INV-6 | `PurgeBarrier::purge` wird nur innerhalb von Schritt 5 gehalten — kein Nesting | ErasureCore |
| INV-7 | Logs/Traces enthalten niemals `EraseTarget`-Klartextinhalt — nur `TargetKind` und count | Alle |
| INV-8 | `Completed` und `Refused` sind terminale Zustände — kein weiterer Übergang außer Idempotenz | ErasureCore |
| INV-9 | `gated_seq` ist monoton steigend — kein Layer darf einen Seq-Wert kleiner als den aktuellen zurückgeben | SagaGate |
| INV-10 | Schlüsselbytes werden mit `zeroize` genullt before drop | SagaVault, FileKeyring |
| INV-11 | `open_key`-Map enthält nie Einträge nach `Completed` | ErasureCore |
| INV-12 | `recover_open_sagas` liefert immer alle nicht-terminalen Sagas nach einem Neustart | SagaJournal |

---

## 13. Recovery-Pfad nach Crash

```
Neustart
  │
  ├─► journal.recover_open_sagas() → [EraseSaga mit letztem Zustand]
  │
  ├─► Für jede Saga mit Zustand Requested/Gated/KeyDestroyed/Purging/Verifying/Pending:
  │     └─► in active_sagas eintragen
  │         └─► beim nächsten run_purge_window() aufgreifen
  │
  └─► Für Sagas mit Zustand Restricted:
        └─► Gate ist bereits gesetzt — ReadGate blockiert Zugriff korrekt
```

**Idempotenz-Garantie:** Da alle Schritte idempotent sind, kann jeder Schritt nach einem Crash wiederholt werden. Der Journal-Zustand ist der Wahrheitsspeicher.

---

## 14. Abhängigkeits- und Migrationsplan (D-E1)

### Aktueller IST-Zustand

```
contextra-ports
  ├── erasure.rs         → ErasureLayer, EraseTarget, LayerScope, PurgeCx, PurgeOutcome
  ├── barrier.rs         → CommitBarrier, PurgeBarrier, Permits
  ├── gate.rs            → ReadGate, GateDecision, KeyRange, KeyBound
  ├── keys.rs            → FileKeyring, RecordSealer, FileKey
  ├── ledger.rs          → LedgerSink, LedgerEntry
  ├── observer.rs        → ErasureObserver, ErasureEvent
  ├── report_signer.rs   → ReportSigner
  ├── scanner.rs         → LayerScanner, LayerWitness, Purger  [KANONISCH für LayerWitness]
  └── erasure_legacy.rs  → #[deprecated] Re-Exports (für D-E1 Transition)
```

### SOLL-Zustand nach D-E1

```
contextra-erasure-spi  [neues Crate, Ring 2.5]
  ├── Alle Erasure-Ports aus contextra-ports migriert
  └── contextra-ports::erasure_legacy entfernt

contextra-ports
  └── Nur noch generische Storage-/Index-/Embedding-Traits
```

> [!NOTE]
> D-E1 ist ein **geplanter Folgeschritt**. Für die aktuelle Implementierung: Alle Implementierer importieren aus `contextra-ports` (nicht aus `erasure_legacy`). Die `#[deprecated]`-Annotierungen dienen als Tracking-Marker für die spätere Migration.

---

## 15. Testpflichten

Jede Seam-Implementierung muss folgende Tests bereitstellen:

| Test-ID | Beschreibung | Kategorie |
|---------|-------------|-----------|
| T-J1 | Journal-Append + Recovery liefert korrekte offene Sagas | Unit |
| T-J2 | Nach `Completed`-Append: `recover_open_sagas` enthält Saga nicht mehr | Unit |
| T-G1 | `add_gate` unter gleichzeitigen Commits: `gated_seq` ist monoton | Integration |
| T-G2 | `decision_for_target` liefert `Erased` nach `add_gate` | Unit |
| T-V1 | `destroy_highest_key` → `verify_key_absent` = `true` | Unit |
| T-V2 | Zeroize: `FileKey`-Drop löscht Bytes (via Canary-Pattern) | Unit |
| T-P1 | `purge` mit gleichem Scope zweimal → kein Fehler, kein Doppellöschen | Unit |
| T-P2 | `purge` mit überschrittener Deadline → `Err("deadline exceeded")` | Unit |
| T-W1 | `witness` nach `purge` → `residue_matches == 0` | Integration |
| T-W2 | `witness` ohne `purge` → `residue_matches > 0` | Integration |
| T-L1 | Ledger-Hash-Kette: drei Einträge verifiziert | Unit |
| T-R1 | Vollständige Saga Requested→Completed: alle 6 fsync-Ankerpunkte vorhanden | Integration |
| T-R2 | Saga nach Crash in `Purging`: Recovery → `run_purge_window` → `Completed` | Integration |
| T-C1 | `CommitBarrier`: exclusive wartet auf alle shared vor Rückgabe | Loom |
| T-C2 | `PurgeBarrier`: purge wartet auf compaction | Loom |
