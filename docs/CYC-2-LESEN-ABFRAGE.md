# CYC-2 — Lesezyklus und Abfragezyklus

Ersetzt R14 §6, §11.10, §11.11. Abhängigkeiten: SYS-2 (Run, Siegel), SYS-3 §5, SYS-4, CYC-3 (Gate), CYC-7 (Projektion, Partitionen), CYC-8 (Branches). Gate: **G-R** (Kern), **G-R3** (Query-Plan).

## 1. Zweck und Grenzen

Zwei Ebenen mit einer Schnittstelle:
- **Lesezyklus (Kern, `storage`)**: Views, Punktlesen, Cursor, Page, Body, Cache, Branch-Auflösung. Liefert nie Daten eines gegateten Clusters.
- **Abfragezyklus (`engine::query`)**: Query-Plan QP-1…QP-9 über mehrere Kanäle, Fusion, Tokenbudget. Nutzt Kern-Views, Projektionen und `filter_visible`.

Nicht Teil: Aufbau der Indizes (CYC-7), Löschung (CYC-3).

## 2. Views, Gate-Reihenfolge, Lease

**View-Erzeugung** (`INV-VIEW-2`): `visible_stamp` laden (Acquire), dann `Arc<Version>`. `view_durable()` begrenzt auf `durable_stamp`. `view_at(S)`: `S < history_floor` ⇒ `BelowFloor{floor}`; `S > visible_stamp` ⇒ `InvalidInput`.

**Gate zuerst.** Jede Leseoperation prüft vor jeder Entsiegelung, in dieser Reihenfolge:
1. Clusterzustand: unbekannt ⇒ `UnknownCluster`; `Retired`, `Shredded`, `Purged` ⇒ `Erased{at:S}` **für jede Sicht, auch ≥ S** (Erasure-Barriere überstimmt Snapshot-Isolation, `INV-ERA-1`).
2. `erased_ranges`: Sicht `V < erase_stamp` in einem gelöschten Bereich ⇒ `Erased{at}` (`INV-ERA-5`). Für `V ≥ erase_stamp` sind Versionen mit `stamp ≤ erase_stamp` im Bereich unsichtbar; Versionen danach sichtbar.
3. `pending_erase` (RAM, vergiftetes Handle) ⇒ `ErasePending`.
4. Restriction `BLOCK_READ` ⇒ `Restricted{BLOCK_READ}`; zweckgebundene Lesung nur mit nicht klonbarer `RestrictedReadCap` (Observer-Ereignis, nie in Cache/Feed).
5. Widerruf: `Revoked`; Lease abgelaufen ⇒ `SnapshotTooOld{stamp, floor_hint}`.

`SnapshotTooOld` (Lease) und `BelowFloor` (Stamp nicht mehr vorhanden) sind getrennte Fehler.

**Lease.** `[view_lease]`, verlängerbar per `renew()` bis `[view_lease_max]` ab Erzeugung (monotone Uhr). Nach Ablauf wirkt die View wie widerrufen, spätestens `[revoke_grace]` danach hält sie keine `Arc<Version>` mehr (`INV-VIEW-3`). Lange Operationen setzen über `ResumeToken` + `view_at(V)` fort und halten dafür einen `Hold` (≤ `[max_hold_ttl]`). Erase widerruft unabhängig von der Lease; `ViewPolicy::Wait(t)` wartet höchstens `min(t, Lease-Ende)`.

**`filter_visible(&[RecordRef], VisCheck)`** prüft Clusterzustand (Schritte 1–5), Restriction und Stamp. `VisCheck::DocHead`: ein Chunk ist sichtbar, wenn seine Version dem `head` der Sicht entspricht (RAM-Vergleich über gecachte Head-Tabelle, Punktlesen bei Cache-Miss, CYC-7 §4). Jedes Extension-Ergebnis MUSS ihn passieren (`INV-VIS-1`).

## 3. Lesepfad

### 3.1 Punktabfrage

Suchschlüssel `(cluster, branch, key, tag_max = (V << 8) | 0xFF)`; gesucht wird der erste Eintrag mit `tag ≤ tag_max` in der Reihenfolge `cmp`.

1. Quellen in Reihenfolge neu → alt: Memtable → immutable Memtables → Staging-Runs (nur bei `ClusterFilter`-Treffer) → je Level die Dateien der Wartungsgruppe (Binärsuche über Dateibereiche, `ClusterFilter`).
2. Je Datei: `ClusterDirectory` (RAM-Sparse-Index, höchstens **eine** Seite) → Abschnitt → `BranchTable` → `FilterBlock` → `SubIndex` → **ein** Datenblock je Treffer-Level.
3. **Auflösung.** Sei `p` der neueste sichtbare Punkteintrag (`tag ≤ tag_max`) und `r` der höchste Stamp aller überdeckenden `RangeDelete`-Marker (Stamp `≤ V`, gleicher Branch) aus **neueren oder gleichen** Quellen. Ergebnis: `p`, falls `p.stamp > r` und `p.kind ≠ Delete`; sonst „nicht vorhanden“. Die Suche endet bei der ersten Quelle, in der `p` gefunden wurde, **nachdem** deren `RangeDelBlock` geprüft wurde (auch bei negativem Filter, O(log r)); ältere Quellen können `p` nie überdecken.
4. Blob-Körper (`PutRef`) werden nie bei `get` gelesen; `Entry::body()` lädt `record_uid`-Record, prüft das Gate vor **und** nach dem Entsiegeln.

### 3.2 Cursor

k-Wege-Merge über die Quellen eines Clusters und seiner Branch-Kette (CYC-8 §3). Der Cursor liefert je Schlüssel die aufgelöste Version (Regel 3.1.3), überspringt Tombstones, hält `RangeDelete`-Fragmente und Kappen im Iterator. Der Cursor prüft Gate, Widerruf und Lease **zwischen Blöcken**. Beim Erreichen eines für die Sicht gelöschten Bereichs (Nr. 2 oben) liefert er `Err(Erased)` und ist danach beendet.

### 3.3 Page

`page(range, dir, budget, resume)`:
1. Liest DataBlocks nacheinander über den SubIndex, **nie** Blob-Körper; Kopf und `weights` liegen im Datenblock.
2. **Fortschrittsgarantie:** ein Eintrag, der `max_weight`, `max_bytes` oder `max_entries` sprengt, beendet die Seite, außer die Seite wäre leer (dann wird er geliefert).
3. `weight_sum` aus dem SubIndex ist eine **Obergrenze** (zählt alle Versionen und Tombstones) und nie Korrektheitsquelle. Verwendung: Schnellpfad, wenn `max_weight − weight_used ≥ block.weight_sum` (Block ohne Einzelprüfung des Budgets); `Page.skipped_blocks` zählt Blöcke, die ohne Entsiegeln übergangen wurden, und zwar nur (a) außerhalb des Bereichs laut SubIndex-Schlüsseln und (b) per `BlockPruner::may_skip` (CYC-7 §3). (R14 behauptete das Überspringen über `weight_sum`; das war nur für die beiden genannten Fälle haltbar, GAP-37.)
4. `ResumeToken`: AEAD (XChaCha20-Poly1305) unter einem Prozessschlüssel, enthält `(instance_id, stamp, cluster, branch, letzter Schlüssel, Richtung)`; ungültig nach Neustart (`InvalidInput`), nach Löschung `Err(Erased)`.
5. Richtung vorwärts/rückwärts liefert dieselbe Menge bei gleichem Budget-unabhängigem Bereich (Property-Test).

### 3.4 Block-Cache

Eintrag `(file_id, block_uid) → (cluster, gen, plaintext)`; Treffer nur bei `entry.gen == cluster.gen` (`INV-KEY-2`). Gate und ER2 erhöhen `gen`; Lane `urgent` überschreibt Puffer (`Zeroizing`). **Check-after-decrypt:** `gen` wird vor und nach dem Entsiegeln gelesen; Abweichung ⇒ Ergebnis verwerfen, Gate-Check erneut (liefert `Erased`).

### 3.5 Körper und Schlüssel

`body()`: Gate ⇒ `Erased`; Provider offline ⇒ `KeyUnavailable{retry_after}` (Schlüsselcache gilt bis `[key_grace]`); Schlüssel fehlt ohne Löschnachweis ⇒ `KeyLost` (Quarantäne). Nie Daten, nie „leer“.

### 3.6 Integrität

Ein gelesener Block muss mit der im SubIndex stehenden `block_uid` entsiegeln. Abweichung ⇒ `Corrupt{Block}` und Block-Vergiftung (F-4); gesunde Cluster laufen weiter. Der Footer-Tag wird beim Öffnen der Datei geprüft (SYS-2 §5.4).

## 4. Branch-Auflösung (Chain-Resolve)

Layout: `cmp` sortiert `branch` vor `user_key`; ein Branch ist je Abschnitt ein zusammenhängender Blockbereich (`BranchTableBlock`). Der Filter ist ein Bloom über `(branch, user_key)`, also eine RAM-Probe je Kettenebene.

Punktlesen auf Kette `[b0 = Blatt … bd = Wurzel]` mit Kappen `cap_0 = V`, `cap_{i+1} = min(cap_i, fork_stamp(b_i))`: je Ebene und Datei (a) `BranchTable` negativ ⇒ weiter; (b) bei `has_range_del` die Intervallliste prüfen, auch bei negativem Filter; (c) Filter negativ ⇒ weiter; (d) Lesung nur im Blockbereich des Branches. Erster sichtbarer Treffer `≤ cap_i` beendet; ein überdeckendes `Delete`/`RangeDelete` beendet mit „nicht vorhanden“ ohne Elternlesung. Erwartete Blocklesungen ≈ `1 + d × FPR` (H-21). Der clusterweite `EraseRange`-Maskensatz gilt **vor** der Versionsauflösung (Abschnitt 2 Nr. 2). Cursor: Fan-in = Summe der Kettenebenen mit Treffern im Bereich. Korrektheit: `INV-BR-5` (kein falsches Negativ), Property-Test gegen Referenzmodell inkl. `RangeDelete` auf Zwischenebenen, `Fork At(S)`, Compaction mit Fork-Floor. Verfehlt H-21 bei Tiefe 8 das Ziel, sinkt die harte Obergrenze `max_branch_depth`; das Messverfahren wird nicht gelockert.

## 5. Abfragezyklus (`engine::query`)

Eingabe:

```rust
pub struct Query { pub principal: Principal, pub collections: Vec<CollectionId>, pub text: Option<String>, pub vector: Option<Vec<f32>>,
    pub k: u32, pub budget: TokenBudget, pub min_stamp: Option<Position>, pub at: Option<TimeTravel /* SnapshotSetId | Epoch */>,
    pub order: OrderPolicy, pub egress: Option<EgressTarget>, pub filters: Filters /* Mandant, Zeit, Quelle */ }
pub struct ContextPack { pub items: Vec<ContextItem>, pub tokens_used: [u32; 4], pub omitted: u32, pub snapshot_set: SnapshotSet,
    pub channel_contrib: ChannelContrib, pub trust_labels: Vec<Trust>, pub lineage: Vec<Lineage>, pub trimmed_after_restore: bool,
    pub egress_denied_count: u32, pub time_travel_degraded: bool, pub explain: Explain }
```

| Schritt | Vorgang | Fehler |
|---|---|---|
| QP-1 | `TenantPolicy` und Zugriffskontext prüfen (SYS-5 §3); Anfrage normalisieren: NaN/Inf, Dimension, `k ≤ Limit`, Textlänge | `Unauthorized`, `InvalidInput` |
| QP-2 | `view_durable()` ⇒ `V`. **`SnapshotSet`** bilden: `(instance_id, V)` plus je Kanal/Partition `(partition, generation, applied_stamp)`. Generationen werden gepinnt (`EpochPinRegistry`, Lease ≤ `[snapshot_set_lease_max]`) | |
| QP-3 | Read-your-writes: bei `min_stamp` bis `[ryw_wait]` auf `watermark ≥ min_stamp` warten, sonst Delta-Pfad; unbedeckter Bereich `(applied_stamp, V]` ohne Delta ⇒ `IndexLag` (`INV-RAG-3`) | `IndexLag` |
| QP-4 | Kanäle **parallel** (ComputePool), je mit Gate-/Mandantenmaske **vor** der Exploration: Vektor (Basis ∪ Delta, Modus A/B/C), BM25/WAND, Graph | |
| QP-5 | Kandidaten sammeln, `View::filter_visible` (Gate, Restriction, `DocHead`) batchweise | |
| QP-6 | Fusion (§6) | |
| QP-7 | Budget (§6): `page()`/`DiBud` wählen anhand `weights[W]`; Körper erst für gewählte Treffer, Gate-Prüfung vor **und** nach dem Entsiegeln; Pseudonym-Auflösung (CYC-7 §9) und Restreserve | |
| QP-8 | **Letzte Prüfung vor Ausgabe:** Gate-Generation jedes Treffers erneut (`INV-VIS-1`); Trust-Label und Lineage anfügen; `EgressPolicy` (SYS-5 §5) | |
| QP-9 | Antwort trägt `SnapshotSet`, Kanalbeiträge, `explain` (nur IDs/Scores, nie Inhalte gelöschter Records; nicht länger gecacht, als die Sicht lebt) | |

**Gate × Query × Rebuild (RACE-23):** wird ein Quellcluster zwischen QP-5 und QP-8 gegatet, scheidet sein Treffer in QP-8 aus; hält die Query eine Partitionsgeneration, deren Quelle gegatet wird, wird das `SnapshotSet` widerrufen (`SnapshotRevoked{Erase}`), die Query wiederholt auf `g+1` höchstens `[guard_retry_max]` Mal, danach `IndexLag`.

**Determinismus (`INV-RAG-4`).** Gleiche Anfrage + gleiches `SnapshotSet` ⇒ bitgleiches Ergebnis, unabhängig von Threadzahl und Hashreihenfolge. Konsequenzen: Kanäle liefern Ergebnislisten mit totaler Ordnung `(Score absteigend, DocId aufsteigend)`; Mengenoperationen laufen auf sortierten Vektoren; Zufall nur aus `partition_seed`/injiziertem `Rng`.

## 6. Fusion, Kalibrierung, Tokenbudget

**RRF (Standard).** Je Kanal `c` mit Rang `r_c(d)` (1-basiert, nur aus der Ordnung, nicht aus dem Rohscore): `S(d) = Σ_c W_c · 2^32 / (60 + r_c(d))` in **ganzzahliger** Arithmetik (`u64`, abgerundete Division, `W_c` ganzzahlig, Standard 1). Damit ist die Fusion bitgleich ohne Gleitkommazuordnung. Gleichstand: `S` absteigend, dann `DocId` aufsteigend (`INV-RRF-1`). `k = 60` ist ein Format der Fusion, kein Konfigurationswert (Änderung = Release-Notiz).

**Kalibrierte Fusion (optional).** Nur wenn **jeder** aktive Kanal einen Kalibrator mit ≥ `[k_min]` Beobachtungen je Gruppe hat (Isotonic, Platt oder Conformal, `contextra-rank`), ersetzt `S(d) = Σ_c w_c · p_c(d)` (kanonische Summationsreihenfolge nach Kanal-ID, `f64`) die RRF-Fusion; sonst gilt RRF (deterministische Rückfallregel). Kalibratoren trainieren nur auf aggregierten Scores ohne Nutzer-/Query-Bezug; Persistenz in einem `NonPersonal`-System-Space der Engine.

**`DiBud` (`contextra-rank`, deterministisch, zustandslos):**
1. Kandidaten aus QP-6 mit Zoomstufen (L0 Körper, L1/L2 Kopf, CYC-7 §10) und Gewichten je Stufe.
2. Je Kandidat die höchste Stufe, deren Gewicht in das Restbudget passt (Absenken bei Knappheit).
3. Greedy nach Nutzendichte `score / weight` mit Redundanzabzug (MMR-Ähnlichkeit zu bereits gewählten Items); Gleichstand: `DocId` aufsteigend.
4. Ausgabe `OrderPolicy::ByScore` (Standard) oder `PrefixStable` (Ordnung nach `(Klasse, DocId)`, für prefix-cache-freundliche Prompts; der Cache liegt außerhalb, NZ-18).
5. Ergebnis `ContextPack`.

**Pseudonym-Auflösung.** Gewichte beschreiben den pseudonymisierten Text; die Rückübersetzung ändert die Tokenzahl. `DiBud` reserviert `[restore_slack_fraction]` des Budgets; nach der Auflösung wird die Endlänge geprüft, bei Überschreitung werden Items niedrigster Nutzendichte deterministisch entfernt, das Ergebnis trägt `trimmed_after_restore`. Für externe Ziele (nur Platzhalter) entfällt die Reserve.

## 7. Zeitreise

`Query::at(S)` ist nur möglich, wenn (a) ein `SnapshotSet` benannt ist (Generationen gepinnt, Lease ≤ `[snapshot_set_lease_max]`) oder (b) für jede beteiligte Partition eine retained Generation mit `applied_stamp ≤ S` existiert; sonst Kern-Vollscan-Pfad nur für `Corpus`-Partitionen unter `[scan_fallback_max_vectors]`, sonst `Err(IndexHistoryUnavailable)`. Ein Ergebnis enthält nie Daten gegateter Cluster (Erase überstimmt Zeitreise, `INV-ERA-1`); nach Generationswechsel ist die Zeit vor einer Löschung für die gelöschten Quellen unwiederbringlich; das Ergebnis weist `time_travel_degraded` aus, wenn Quellen fehlen.

## 8. Fehler- und Absturzverhalten

Lesen hat keine Absturzzustände (Sichten sind flüchtig). Fehlerabbildung: SYS-3 §9. Provider-Ausfall ⇒ `KeyUnavailable`, nie leeres Ergebnis. Index-Ausfall einer Partition ⇒ Kanal liefert `Err(Corrupt/IndexLag)`; QP-4 verwirft den Kanal und weist `degraded_channels` im `explain` aus (nicht still). Dass ein Kanal ausfällt, ändert nie die Sichtbarkeitsgarantie.

## 9. Parameter

SYS-4 §2: `view_lease`, `view_lease_max`, `revoke_grace`, `key_grace`, `max_page_bytes`, `ryw_wait`, `rerank_depth`, `k_min`, `restore_slack_fraction`, `scan_fallback_max_vectors`, `snapshot_set_lease_max`, `max_branch_depth`.

## 10. Entscheidungen und geschlossene Lücken

DEC-22, DEC-28, DEC-29, DEC-72 (RRF in Festkomma-Ganzzahlen, `k = 60` Formatkonstante), DEC-73 (Kalibrierte Fusion nur bei vollständiger Kalibrierung), DEC-74 (`weight_sum` nur Schnellpfad); GAP-37, GAP-38 (SYS-7 §3).

## 11. Abnahme

`read_gate_order` (Fehlerreihenfolge §2), `read_erased_view_ge_s`, `erased_range_masks`, `point_get_one_block` (H-2), `cursor_eq_model`, `page_progress`, `page_forward_backward`, `block_cache_gen` (`loom`, RACE-1, RACE-4), `check_after_decrypt`, `branch_resolve_model` (INV-BR-5), `rrf_bit_identical` (Threadzahl variiert), `query_snapshot_determinism`, `gate_x_query_race` (RACE-23), `dibud_budget_never_exceeded`, `restore_slack`, `time_travel_after_erase`. Messung: H-2, H-3, H-4, H-21, H-29, H-30, H-39. Gates G-R, G-R3.

**Offene Punkte:** keine.
