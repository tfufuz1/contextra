# Implementierungsspezifikation `contextra-engine` (Löschkette – Treiber, Resolver, Worker, API)

*Ring 3 · Konzept-WP 5, 9, 10 · Abhängig von: 01–04, 06–09 · Blockiert: 11, 12 · Index: `00-INDEX.md` (K-8, K-12, K-24, K-26, K-43, K-44, A-1)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

Die Engine kennt alle Schichten. Sie (1) löst `EraseTarget` in einen `ResolvedScope` auf, (2) treibt die Saga (`Requested → … → Completed`) über den Kernel-Port und die registrierten Layer, (3) plant G3 gebündelt und gedrosselt in einem Worker, (4) bietet die öffentliche Erase-API als `impl Contextra`, (5) räumt RAM-Caches, (6) ersetzt `drop_collection` und stellt die Altpfade (`delete`) sauber.

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `types`, `ports`, `mvcc`, `store`, `vector`, `text`, `checkpoint`, `rank`, `graph`, `tokio`, `tokio-util` (rt), `tracing`, `parking_lot` (`Cargo.toml`). Alle Schichten sichtbar ✔ |
| `Contextra` | `engine/src/lib.rs:361`: Felder `storage: Arc<LsmStorage>`, `next_tx`, `dimension`, `config`, `collections`, `tenant_collections`, `cancel_token: CancellationToken`, `task_tracker: TaskTracker`, `orphan_registry`, `metrics_sink`, … ; `inner_storage()` (`:~396`); Typalias `TenantCollectionMap` mit `Collection<TenantScopedStorage<Arc<LsmStorage>>>` |
| Kernelzugriff | `LsmStorage::erasure_kernel()` (03 §3, K-24) liefert `Arc<dyn ErasureKernel>` |
| Schlüsselbildung | `Collection::namespaced_key(key, key_type)` (`collection/mod.rs:517-548`): `default` ⇒ Typ 0 `key`, Typ 1 `__docid:{key}`, 2 `__rel:{key}`, 3 `__tx_intent:{key}`, 4 `key`; sonst `prefix ‖ [key_type] ‖ key`. `user_key_prefix()`; Text-Namespace `__txt:{name}:` (`maintenance.rs:868`); Mandantenpräfix `t:{tenant}:` (`store/tenant_codec.rs`) |
| Löschen | `Collection::delete` (`crud/delete.rs`): `#[tracing::instrument(level="trace", skip(self))]` **ohne** `id` (K-43: `id` landet im Span); `delete_op` tombstoned `user_key` und `doc_key`, `stage_text_delete` |
| `drop_collection` | `maintenance.rs:852-873`: `delete_prefix` zweimal, `commit`; HNSW, Graph, Checkpoints, `__col_idx` unberührt (K-44) |
| Worker | `background_workers/{config,expiry_workers,hyperedge_worker,orphan_workers,mod,tests}.rs`: Muster `tokio::spawn`, `interval`, `MissedTickBehavior::Skip`, `CancellationToken`, `MAX_ORPHANS_PER_TICK` |
| Caches | Embedding-, KV-, Query-, Block-Cache: Orte **[?]** (`rg "Cache" crates/contextra-engine/src -l`) |
| Loom | `Contextra` unter `cfg(loom)` nicht gebaut (K-31) |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `src/erasure/mod.rs` | **NEU**: `ErasureRuntime` (hält Kernel, Registry, Worker-Handle, Notify), Re-Exporte |
| `src/erasure/config.rs` | **NEU**: `ErasureConfig` (Konzept 7.5), `EraseOptions` |
| `src/erasure/resolver.rs` | **NEU**: `ScopeResolverImpl` (Planung) |
| `src/erasure/registry.rs` | **NEU**: `LayerRegistry` (Reihenfolge, `expected_layers`) |
| `src/erasure/driver.rs` | **NEU**: `ErasureDriver::run_job` (Saga) |
| `src/erasure/budget.rs` | **NEU**: `EngineIoBudget` (`IoBudget`-Adapter auf Store-Bucket + Druck) |
| `src/erasure/caches.rs` | **NEU**: `ErasureObserver`-Implementierungen, Epoche-im-Schlüssel |
| `src/erasure/api.rs` | **NEU**: `impl Contextra { erase, erase_status, erase_dry_run }` (K-8) |
| `src/erasure/adapters.rs` | **NEU**: `TextTxSource`, `GraphTxSource`-Adapter, `ProgressSink`-Adapter |
| `src/background_workers/erasure_worker.rs` | **NEU** (§8) |
| `src/background_workers/mod.rs`, `config.rs` | Registrierung, Konfiguration |
| `src/collection/crud/delete.rs` | `skip(self, id)` (K-43, G-3); sonst Retire unverändert |
| `src/collection/maintenance.rs` | `drop_collection` ⇒ `erase(Collection)` (§10) |
| `src/collection/crud/{insert,update,kv}.rs`, `collection/mod.rs` | Gate-Fehlerabbildung `Erased` (§9) |
| `src/lib.rs`, `contextra_impl/{lifecycle,wiring,collections}.rs` | Feld `erasure: ErasureRuntime`, Start/Stop (§11) |
| `src/export.rs`, `src/import.rs` | Gate-Filter bei Export, Ablehnung gesperrter Importe (§9) |

## 4. Konfiguration und Optionen

```rust
pub struct ErasureConfig {                   // Konzept 7.5, unverändert
    pub coalesce_window: Duration,           // 5 min
    pub sla: Duration,                       // 24 h
    pub erase_io_share: f32,                 // 0.5
    pub max_units_per_tick: usize,           // 1_024
    pub retry_backoff: (Duration, Duration), // 1 s .. 5 min, Jitter über Rng-Port
    pub quarantine_after_attempts: u32,      // 10
    pub checkpoint_policy: CheckpointTaintPolicy, // Drop
    pub checkpoint_grace: Duration,          // 0
    pub max_pin_age: Duration,               // 7 d
    pub profile: ErasureProfile,             // Standard | Regulated
    pub recheck_interval: Duration,          // 5 min (grober Timer)
}
pub struct EraseOptions { pub reason: ErasureReason, pub await_level: AwaitLevel /* Default Gated */, pub policy: Option<ErasurePolicy> }
pub struct EraseHandle { pub id: ErasureId, pub reached: Guarantee, pub state: ErasureState }
```
`Default` für `ErasureConfig` existiert (außer `profile`-abhängigen Pflichtwerten); Zeit nur über `Clock`, Jitter nur über `Rng` (`Rng::fill_bytes`, K-12), **keine** `HashMap` (`BTreeMap`/`ahash`).

## 5. Scope-Auflösung (`resolver.rs`)

Planung ist **read-only und deterministisch**; `erase_dry_run` nutzt denselben Code und liefert nur Mengen. Die Auflösung liest über **dasselbe** `TenantScopedStorage` wie die Collection (K-6 aus Konzept 12.1 Nr. 6: alle Engine-Pfade gehen durch `TenantScopedStorage`; **[?]** Gegenprüfung `rg "inner_storage\(\)" crates` auf Pfade, die ohne Mandantenpräfix schreiben).

### 5.1 `EraseTarget::Document { tenant, collection, key }`

1. Collection laden (`tenant_collections`); fehlt sie ⇒ `Err(NotFound)`.
2. `doc_id = DocId::from_key(key)`. **Schlüsselverifikation:** Reverse-Lookup `__docid:{doc_id}` lesen und mit `key` byte-gleich vergleichen. Ungleich ⇒ `ErasureError::ScopeAmbiguous` (nie Über-Löschung, B-13). Fehlt der Reverse-Lookup (Dokument existiert nicht): `Err(InvalidTarget("document not found"))`.
3. **Schlüsselsicht** (`KeyScope`, Rohschlüssel mit `t:{tenant}:`-Präfix aus dem Mandantenscope):
   - `Exact(user_key = namespaced_key(key, 0))`, `Exact(doc_key = namespaced_key(doc_id_le, 1))`;
   - Text: `Exact(__txt:{name}:dl:{doc})`, `Exact(__txt:{name}:fw:{doc})` (Format über `InvertedIndex::key_with_id`, nicht duplizieren: der Resolver ruft eine `pub`-Hilfe des Text-Crates `InvertedIndex::scope_keys_for(doc)` **[?]** neu, siehe 06 §3);
   - Relationen: `namespaced_key(…, 2)`-Schlüssel mit `doc`/`key` im Schlüsselkörper **[?]** (`rg "key_type: 2|, 2\)" collection/relate.rs`), Intent-Schlüssel (Typ 3) nur falls offen;
   - Graph: `Exact(__graph:prov:{collection}:{doc})` (08 §4).
   - **Nicht** im Schlüsselscope: Postings `i:`/`plb:` (K-30, Text-Layer) und Graph-Objekte (K-38, Graph-Layer).
4. **Dokumentsicht:** `DocScope::Set{tenant, collection, docs={doc_id}}`.
5. `unprovenanced`: Anzahl nicht belegbarer Graph-Objekte (08 §4) für das Dokument.

### 5.2 `EraseTarget::Collection { tenant, collection }`

`collection == "default"` ⇒ `InvalidTarget` (K-5, schon in `validate`). Schlüsselsicht (alle Regeln `In`): `Prefix(t:{tenant}:{collection.prefix})`, `Prefix(t:{tenant}:__txt:{name}:)`, `Exact(__col_idx…)` für die Collection-Registrierung (K-44, Format **[?]** `rg "__col_idx" crates`), `Prefix(t:{tenant}:__graph:prov:{name}:)`. Dokumentsicht: `DocScope::Collection`. Checkpoints: `TaintQuery` mit der Collection (09 §5.1).

### 5.3 `EraseTarget::Tenant { tenant }`

`Prefix(t:{tenant}:)` `In`. Dokumentsicht `DocScope::Tenant`. Komplement-Fälle (Regelmenge, Longest-Match, Index §4): Wo ein globaler Schlüssel (ohne `t:`-Präfix, z. B. Legacy-Default-Collection ohne Mandant) berührt sein könnte, wird er **nicht** eingeschlossen. Ergebnisprüfung (Pflicht): jeder `In`-Schlüssel trägt das Mandantenpräfix (`starts_with(b"t:{tenant}:")`), sonst `ScopeViolation` + Abbruch + Alarm (I-8).

### 5.4 `plan_hash` und Persistenz

`ResolvedScope::new(...)` berechnet `plan_hash` (02 §5). Vor `accept` gilt: `KeyScope::new` ≤ 4 096 Regeln/1 MiB; **große Collection-/Tenant-Scopes sind Präfixe** (wenige Regeln); nur Dokument-Erase hat Exaktschlüssel. Überschreitung ⇒ `InvalidTarget("scope too large")`. Schlüsselbytes stehen im Journal (03 §5), nie in Logs/Metriken (HMAC-Kürzel, I-12).

## 6. Saga (`driver.rs`)

`ErasureDriver::run_job(id)` ist **wiederaufnehmbar** und liest den Zustand aus `kernel.open_jobs()`/`status`. Jeder Schritt ist idempotent. Kein Lock über einen Layer-Aufruf (Rang 150 wird nur im Kernel gehalten, Index §4).

```text
erase(target, opts):
  validate(target)                         // types
  id = new ErasureId (16 Byte via Rng::fill_bytes)   [K-12]
  plan = resolver.resolve(target)          // §5
  receipt = kernel.accept(req, plan)       // Requested → Planned (durable)
  gate = kernel.publish_gate(id)           // Gating → Gated (G1)
  for obs in observers: obs.on_erased(Gated)     // RAM-Caches (Observer-Fehler nie blockierend)
  for layer in registry.layers_for(plan): layer.on_gate(plan)   // RAM-Maskierung, parallel möglich
  if opts.await_level == Accepted/Gated: return EraseHandle (G1) ; Rest läuft im Worker
  else await G3

drive_g3(id):                                // im Worker, gedrosselt
  state == Gated | Purging | Failed(retry):
    for layer in registry.order():           // Text → Graph → Vektor
        outcome = layer.purge(plan, cx{cursor from kernel.status})   // nur committete Arbeit
        kernel.record_layer(id, layer.layer(), outcome.progress)      // durable (Write-Ahead)
        wiederholen bis cursor == None oder Budget/Cancel
    outcome = kernel.purge_store(id, cx)     // Memtable, Erase-Compaction, WAL-Schnitt, Pending-Delete
    checkpoint_layer.purge(...)              // zuletzt (09)
  state == Verifying:
    witnesses = [scanner.scan(...) for layer ∈ expected_layers] + kernel.store_scanners()
    if any remaining > 0: kernel.fail/purge zurück: attempt++ (max quarantine_after_attempts)
    kernel.seal(id, witnesses)               // Ledger-Eintrag
  state == Sealed:
    if checkpoint_layer.remaining_tainted() == 0 || legal_hold_only:
        kernel.complete(id)                  // Gate-Lebensende (I-9)
        observers.on_erased(Completed)
```

**Reihenfolge (Index §4):** (1) RAM-Caches über `ErasureObserver`, (2) Text → Graph → Vektor, (3) Store (`purge_store`), (4) Checkpoints. Begründung: Text braucht `fw:` (privilegiert), Graph braucht den Provenienzindex im Store, beide **vor** dem Store-Purge; Checkpoints zuletzt, weil ihre Klone Daten auf Dateiebene halten und der Store-Scanner sie mit prüft.

**`expected_layers`** = genau die registrierten Schichten des Plans (Text je betroffener Collection **einmal** als `TextIndex`, `CsrGraph`, `HnswIndex`, Store-Schichten, `Checkpoints` falls vorhanden). Eine registrierte, aber leere Schicht meldet `remaining = 0`. Nicht registrierte Schicht ⇒ `NotMeasured{layer}` im Seal (senkt Garantie, I-6).

**`ProgressSink`-Adapter:** implementiert `ports::ProgressSink::record` über `kernel.record_layer` und `register_collapse` über `KernelProgress { kernel, id }` (A-1: `ErasureKernel::register_collapse(id, keys)`).

**Fehlerbehandlung** (`LayerErrorKind`): `Retryable` ⇒ `kernel.fail(id, Some(layer), Retryable)` ⇒ Backoff mit Jitter; `Permanent`/`ScopeViolation` ⇒ ab `Gated` `Quarantined`, vor `Gated` Abbruch (Auftrag endet, Gate nie veröffentlicht); `Poisoned` ⇒ Auftrag bleibt `Gated`, `HealthState` degradiert (03 §5.3); `Cancelled` ⇒ unverändert, Wiederaufnahme beim nächsten Tick.

**SLA:** bei 80 % der SLA verdoppelt sich `erase_io_share`, bei 100 % Metrik/Alarm; `Critical`-Druck wird nie ignoriert (§8).

## 7. Lesepfad und Hydration

- **Punkt-Get/Scan:** Gate im Store (03 §6.3), Engine muss nichts tun. `Collection::get` mappt das Gate-Ergebnis „nicht vorhanden“ (Lesen) bzw. `ContextraError::Erased` (Schreiben).
- **Mehrschritt-Pfade** (Suche → Hydration, `collection/search/{hydrate,hybrid,filtered}.rs`): Epochenprotokoll `e1 = gate.epoch()`, Kandidaten (Text/Vektor/Graph, je mit eigenem `erased_docs`-Filter), Hydration (`hydrate.rs`) prüft `gate.check_doc(tenant, collection, doc)` und liest über den Store (Gate dort erneut); `e2 = gate.epoch()`; bei `e1 != e2` den gesamten Pfad **neu** (max. 3 Wiederholungen, dann `Err(Retryable)`). Ohne aktive Gates ein Acquire-Load (Index DoD 4).
- **Zeitreise** (`temporal_filter.rs`, `get_at_seq`): gleiche Prüfung, Gate gilt für alle Sequenzen.
- **Caches** (§9a): Embedding-, Query-, KV-Cache tragen `gate_epoch` im Schlüssel; ein Eintrag mit älterer Epoche gilt als Miss. Zusätzlich `ErasureObserver::on_erased(Gated)` räumt sofort (nicht Teil von G1, 02 §4.8).

## 8. Worker (`erasure_worker.rs`)

Nach dem Muster in `orphan_workers.rs`, aber **ereignisgesteuert**.

```rust
pub fn start_erasure_worker(rt: Arc<ErasureRuntimeInner>, cancel: CancellationToken) -> JoinHandle<()>;
```
- Schläft auf `tokio::select!{ notify.notified(), sleep_until(next_due), cancel.cancelled() }`. **Kein** `interval`-Polling bei leerer Jobliste (Index DoD 4, Leerlauf-CPU 0). `next_due` = frühester Zeitpunkt aus `coalesce_window`/Backoff/SLA aller offenen Jobs; ohne offene Jobs `None` ⇒ nur `notified`/`cancelled`. Zusätzlich `recheck_interval` (Default 5 min) **nur solange** Jobs offen sind.
- `Notify`-Quellen: `erase()` (neuer Job), Boot (offene Jobs aus `kernel.open_jobs()` sofort), `PendingDelete`-Notify (`pending_delete_notify()`, 03 §8), Druckwechsel Critical→Normal.
- **Prioritätsklassen:** (1) Interaktiv: G1 (`accept`/`publish_gate`) läuft **im Aufruferkontext** von `erase()`, nicht im Worker, nie gedrosselt; (2) Fällig: G3 nach Ablauf des Koaleszenzfensters; (3) Wiederholung nach Fehlern mit Backoff.
- **Koaleszenz:** Alle Jobs, die im Fenster `[first_due, first_due + coalesce_window]` fällig werden, teilen **eine** `purge_store`-Runde (Vereinigung der Hüllen), **einen** WAL-Schnitt, **einen** HNSW-Rewrite pro Index. Der Treiber ruft `kernel.purge_store` mit einer Job-ID, der Kernel bündelt intern über offene Jobs (03 §7.3). Der Worker wartet die Fensterzeit nur ab, wenn kein Job `sla`-gefährdet ist.
- **Budget (`budget.rs`):** `EngineIoBudget: IoBudget` adaptiert den Store-Token-Bucket (`IoBudget`, 03 §7.6) und `SystemPressureMonitor` (`pressure_rx.borrow()`): `Critical` ⇒ `acquire` verweigert/`pressure() = Critical` ⇒ Worker pausiert mit Schlaf+Jitter (Clock-basiert), `Elevated` ⇒ halbes Budget. **G1 nie gedrosselt.**
- **CPU:** Layer rufen `spawn_blocking` selbst; der Worker prüft `cancel` nach jedem Layer-Aufruf und alle `max_units_per_tick` Einheiten.
- **Platz-Preflight vor `purge_store`:** Summe der Kandidaten-SSTable-Größen ≤ freier Platz − `reserve`; sonst `Err(InsufficientSpace)`, Auftrag bleibt `Gated`, Metrik/Alarm.
- **Wartung bei offenen Jobs:** `SnapshotRegistry::reap_expired(ReapPolicy{max_pin_age, ..})` (04 §4) höchstens einmal je Wiederholungsrunde.
- **Abschaltung:** `cancel.cancelled()` ⇒ aktuellen Layer-Aufruf cancel-safe beenden, Fortschritt ist durable; `task_tracker` wartet. Kein neuer Job nach Abschaltung (`erase()` ⇒ `Err(ShuttingDown)`).

## 9. Schreibpfade und Import/Export

- **Insert/Update/KV auf gesperrte Schlüssel:** Der Store lehnt mit `ContextraError::Erased` ab (K-6). Die Engine reicht den Fehler **unverändert** weiter (kein Retry, kein „not found“). `ReinsertPolicy::Deny` (Default) gilt bis `Sealed`; `AllowAfterSealed` erlaubt nach `Sealed` nur `seq > up_to` (03 §6.1). Test TST-ENG-ERA-8.
- **`export.rs`:** Gate-Filter beim Lesen (Erased/Restricted ⇒ überspringen); ein Export während eines laufenden Auftrags enthält den Scope **nicht**. Externe Kopien bleiben `OutOfScopeExternalCopies` im Seal.
- **`import.rs`:** Import von Dokumenten mit gesperrtem Schlüssel ⇒ `Erased`; Import über `restore`-ähnliche Pfade nutzt dieselbe Prüfung.
- **Commit-Intents / Altwerte (B-18):** `record_keys_with_old_values` hält Altwerte nur bis Commit/Rollback; nach Abschluss `zeroize` der Puffer **[?]** Lebensdauer prüfen (`rg "record_keys_with_old_values" crates`), Ergebnis in 03/10 nachtragen.
- **Telemetrie (G-3):** `#[instrument]` ohne `skip` auf Funktionen mit `id`/`key`/`target` ist verboten. `Collection::delete` (K-43): `#[tracing::instrument(level = "trace", skip(self, id))]`; weitere Treffer per `tools/erasure_gates.py` (Index §4, G-3).

## 9a. RAM-Caches (`caches.rs`)

| Cache | Ort | Maßnahme |
| --- | --- | --- |
| Embedding-/Query-/KV-Cache | **[?]** `rg "struct.*Cache" crates/contextra-engine/src` | Implementiert `ErasureObserver::on_erased`; Schlüssel trägt `gate_epoch` |
| Block-Cache | Store (03 §7.5) | Räumen beim Manifest-Swap (Store) |
| `Collection.index` (`DocIndex`), `kv_locks` | Collection | Scope-Dokumente entfernen (`Collection`-Methode `forget_docs(&BTreeSet<DocId>)`, **NEU**, in `maintenance.rs`) |
| Decay/Hyperedge-Worker-Queues | `decay_controller.rs`, `hyperedge_worker.rs` | Einträge für Scope-Dokumente verwerfen (`on_erased`), sonst Wiederauferstehung durch Worker-Rückschreiben |

Observer-Fehler: Zähler + `warn`, **nie** Abbruch (02 §4.8).

## 10. `drop_collection` und `delete`

- `Collection::drop_collection` bleibt als Methode, ruft aber **nicht mehr** `delete_prefix`. Neues Verhalten: `Contextra::drop_collection(name, tenant)` (in `contextra_impl/collections.rs`) ⇒ `erase(EraseTarget::Collection{..}, EraseOptions{ reason: OperatorAction, await_level: Gated, .. })`. Danach wird die Collection aus `collections`/`tenant_collections` entfernt (nach `Gated`, damit kein neuer Schreibzugriff durchkommt). `default` bleibt `Err(invalid_input)` (heute `maintenance.rs:855`).
- **Rückwärtskompatibilität:** `drop_collection` blockiert bis G1 (statt bis zum Tombstone-Commit). G3 läuft im Hintergrund; `erase_status` zeigt Fortschritt. Der MCP-Aufruf (`contextra_drop_collection`, 12) meldet `purged: false` bis `Completed`.
- `Collection::delete` bleibt **Retire** (G0). Profil `Regulated` + `auto_erase_on_delete = true` (ADR-F, Default aus) ruft nach erfolgreichem Retire `erase(Document)` mit `reason: UserRequest` (asynchron, Fehler zählen nur, blockieren das Retire nicht).
- `delete_prefix` (B-07) bleibt für Nicht-Erase-Zwecke, wird aber von Store (03 §7.5) streamend implementiert.

## 11. Verdrahtung und Lebenszyklus

- `Contextra` erhält `erasure: ErasureRuntime` (Typ in `erasure/mod.rs`). `ErasureRuntime::start(storage: &Arc<LsmStorage>, registry, cfg, clock, rng, cancel, task_tracker)`:
  1. `kernel = storage.erasure_kernel()` (K-24);
  2. Observer registrieren (Caches, Collections);
  3. `kernel.open_jobs()` ⇒ offene Aufträge: **vor** Verkehr `on_gate` für jeden offenen Job auf alle Layer (RAM-Maskierung nach Neustart, Index Start-Reihenfolge Nr. 6);
  4. Worker starten, `notify.notify_one()` bei offenen Jobs.
- **Layer-Registrierung:** `LayerRegistry` wird beim Anlegen/Öffnen jeder Collection ergänzt (`TextErasure`, `HnswErasure`, `GraphErasure`, `CheckpointErasure`) und bei `drop` entfernt. Jede Registrierung ist deterministisch geordnet (`BTreeMap<(LayerOrder, CollectionName), Arc<dyn ErasureLayer>>`).
- `Contextra::shutdown`/`close` (`lifecycle.rs`): zuerst `erasure.stop()` (Worker beenden, keine neuen Jobs), dann Collections/Storage.
- `ContextraConfig` (`lib.rs:215`) erhält `erasure: ErasureConfig` (Default); Facade (11) setzt Profilwerte.

## 12. API (`api.rs`, `impl Contextra`, K-8)

```rust
impl Contextra {
    /// G1 sofort; `await_level = Purged` wartet bis G3. Idempotent über `id` (nur intern: neue ID je Aufruf).
    pub async fn erase(&self, target: EraseTarget, opts: EraseOptions) -> Result<EraseHandle, ContextraError>;
    pub async fn erase_status(&self, id: ErasureId) -> Result<ErasureStatus, ContextraError>;
    /// Gleicher Planer, keine Wirkung. Liefert Mengen (Zählungen), keine Schlüssel.
    pub async fn erase_dry_run(&self, target: EraseTarget) -> Result<ErasePlanSummary, ContextraError>;
}
pub struct ErasePlanSummary { pub rules: u32, pub docs: u64, pub unprovenanced: u64, pub layers: Vec<String>, pub est_bytes: u64, pub checkpoints: u32 }
```
Fehlerabbildung `ErasureError → ContextraError`: `InvalidTarget` → `invalid_input`; `ScopeAmbiguous`/`IdempotencyConflict` → `Conflict`-nahe Variante **[?]** (`error.rs` auf vorhandene wählen, sonst `invalid_input` mit fester Meldung); `Busy`, `InsufficientSpace`, `Io` → `Storage`/retryable; `Erased` bleibt `ContextraError::Erased`. `erase_status` für unbekannte ID ⇒ `NotFound`.

**Facade-Invariante (K-8):** Die `contextra`-Crate hat ≤ 20 `pub fn` (✔ Test-Invariante in `contextra/src/lib.rs:3`). Die drei Methoden stehen als `impl Contextra` **hier**; die Facade fügt nur die Typ-Re-Exporte hinzu (11).

## 13. Tests

| ID | Inhalt |
| --- | --- |
| TST-ENG-ERA-1 | Resolver Document: Schlüsselmengen exakt wie §5.1 (Golden), Hash-Kollision ⇒ `ScopeAmbiguous` (Test mit künstlicher Kollision über Mock-`DocId::from_key`) |
| TST-ENG-ERA-2 | Resolver Collection/Tenant: alle `In`-Schlüssel tragen Mandantenpräfix; fremder Mandant bit-identisch nach Erase (C3) |
| TST-ENG-ERA-3 | Saga End-to-End mit Mock-Layern: Zustandsfolge, Reihenfolge (Text, Graph, Vektor, Store, Checkpoints) |
| TST-ENG-ERA-4 | Crash an jedem Schritt (`CrashVfs`, CP-1…CP-12) ⇒ Wiederaufnahme beendet im selben Endzustand wie ohne Crash |
| TST-ENG-ERA-5 | `expected_layers`: nicht registrierte Schicht ⇒ `NotMeasured` im Seal, `Guarantee.purged = false` |
| TST-ENG-ERA-6 | Worker Leerlauf: Tokio-Zeit pausiert, 1 h virtuell ohne Jobs ⇒ 0 Polls (Zähler am `Notify`/Timer) |
| TST-ENG-ERA-7 | Druck: `Critical` ⇒ G3 pausiert, G1 trotzdem sofort |
| TST-ENG-ERA-8 | Insert auf gesperrten Schlüssel ⇒ `Erased`; nach `Sealed` + `AllowAfterSealed` ⇒ Insert ok, alte Version via `get_at_seq` weiter gesperrt |
| TST-ENG-ERA-9 | Mehrschritt-Suche: Gate-Veröffentlichung zwischen Kandidaten und Hydration ⇒ Dokument erscheint nie (Loom-Modell, 13 §8) |
| TST-ENG-ERA-10 | `drop_collection` ⇒ HNSW, Graph, Checkpoints, `__col_idx` bereinigt (Witness 0) |
| TST-ENG-ERA-11 | Koaleszenz: 10 Dokument-Aufträge im Fenster ⇒ **ein** Erase-Compaction-Lauf (Zähler am Kernel) |
| TST-ENG-ERA-12 | `rg "block_on" crates/contextra-engine/src/erasure crates/contextra-engine/src/background_workers/erasure_worker.rs` leer (G-1) |
| TST-ENG-ERA-13 | Canary `T-DEL-1` (13 §7) |
| TST-ENG-ERA-14 | Boot mit offenem Job: Layer-`on_gate` vor erstem Lesezugriff (Reihenfolge-Test) |

## 14. Arbeitspakete

| WP | Inhalt | Abhängig |
| --- | --- | --- |
| 10.1 | `config`, `registry`, `adapters`, `budget` | 03, 02 |
| 10.2 | `resolver` (Document zuerst), `erase_dry_run` | 10.1 |
| 10.3 | `driver` + `api` (G1-Pfad), Boot-Wiederaufnahme | 10.2 |
| 10.4 | Worker (G3, Koaleszenz, Druck) | 10.3, 03 WP-4 |
| 10.5 | Caches/Observer, Epoche im Schlüssel, `forget_docs` | 10.3 |
| 10.6 | Collection-/Tenant-Resolver, `drop_collection` → `erase` | 10.4 |
| 10.7 | Text/Vektor/Graph/Checkpoint-Registrierung (je nach 06–09) | 06–09 |
| 10.8 | Telemetrie-Korrektur (`delete.rs` G-3), Export/Import-Gates | 10.3 |

**Erste Auslieferung** = 10.1–10.4 + 10.6 (LSM, Dokumentsicht, Canary grün). Fehlende Layer senken die ausgewiesene Garantie (I-6).

## 15. Definition of Done

Canary grün; Crash-Matrix grün; Leerlauf-CPU 0; Gates G-1…G-6 grün; `tests/layering.rs` grün; keine neue Kante in `capabilities.toml`.

## 16. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-E1 | Cache-Orte und Typen | `rg` siehe §9a |
| O-E2 | Relationen-Schlüssel (`__rel:`) enthalten Doc-Bezug? | Lesen `collection/relate.rs` |
| O-E3 | Format `__col_idx` | `rg` siehe §5.2 |
| O-E4 | Fehlerabbildung `ScopeAmbiguous` | siehe §12 |
| O-E5 | Lebensdauer `record_keys_with_old_values` | siehe §9 |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `engine/src/erasure/` (`driver.rs` 524, `api.rs` 233, `resolver.rs` 146, `registry.rs` 101, `caches.rs` 93, `budget.rs` 77, `config.rs` 56, `adapters.rs` 39, `types.rs` 31, `mod.rs` 175) und `background_workers/erasure_worker.rs` (36) ✔. Der Worker ist ereignisgetrieben (Leerlauf-CPU 0 ✔). **Keine Treiber-Tests** (`run_pass`, `drive_job`) — deshalb blieben K-45…K-47 unentdeckt.

**Korrigierter Treiberablauf (ersetzt den Ablauf in §6; normativ).**

```text
drive_job(job):
  1  Requested|Planned|Gating  → publish_gate → mask_layers (Fehler ⇒ unmasked, Retry) → Gated
  2  Quarantined               → return Quarantined           (nur Kernel-Zustand zählt)
  3  Gated|KeysDestroying|Failed|Purging:
       kernel.begin_purge(id)                                  ← A-4
       genau EIN Tick: nächster fälliger Layer (Reihenfolge Text → Graph → Vektor), sonst Store, sonst Checkpoint
       record_layer / StoreProgress                            → return Progress
  4  alle Layer + Store erledigt → Verifying: Zeugen sammeln
       Rest > 0 → fail(layer, Retryable)   (Kernel verwirft Layer-Cursor, §3.4)
       sonst    → seal
  5  Sealed → complete → Observer(Completed)
Jeder Fehler: fail(...) auswerten (Quarantined|Failed|Err), niemals verwerfen.
```

**Ergänzungen.**

* **K-52 (A-7):** `ErasureId`-Bytes über `KeyMaterialSource`; `SeededRng`-Fallback (`mod.rs:93-98`) nur in Tests. Backoff-Jitter darf PRNG nutzen.
* **K-53:** Observer-Tasks über `TaskTracker`, `stop()` wartet bis `shutdown_grace`; Replay von `Gated` beim Start (14 §3.8).
* **K-56:** `tenant_of` ohne Fallback `SYSTEM`; leerer Scope ⇒ `InvalidTarget` in `accept`.
* **K-57:** ein Tick je Job und Pass, `DriveOutcome::Progress`, `cfg.max_jobs_per_pass`, SLA-Jobs zuerst; SLA-Überschreitung ⇒ Laufzeitfeld `sla_breached_at` im Status.
* **K-48 (A-6):** `required_layers`, `LayerMissing`; `Contextra::build` registriert Layer (11). 
* **A-8:** `Contextra::erase_resume(id)`; kein MCP-Werkzeug.
* **K-55/K-59:** `reap_expired`-Ergebnis wird erfasst; die G-2-Baseline-Einträge für `engine/src/erasure/driver.rs` (5) und `mod.rs` (1) entfallen.
* **Tests:** `TST-DRV-ORDER-1`, `-FAIL-1/2`, `-RESET-1`, `-GATEERR-1`, `-SCHED-1`, `TST-ENG-WIRE-1`, `TST-ENG-NOTREADY-1`, `TST-ENG-ID-1`, `TST-OBS-REPLAY-1` (14 §6), je gegen den **echten** `StoreErasureKernel` mit `MockLayer`, zusätzlich gegen `MockErasureKernel`.
