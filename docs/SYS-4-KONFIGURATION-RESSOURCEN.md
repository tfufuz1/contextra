# SYS-4 — Konfiguration, Ressourcen, Zahlenregister

Ersetzt R14 §3.4, §3.7, §15. Abhängigkeiten: SYS-1 (Ports, Threads), SYS-2 (Formate). Gate: G-0. **Dies ist die einzige Stelle, an der Zahlenwerte stehen** (Index §1). Andere Dateien zitieren einen Wert nur per Namen in der Schreibweise `[name]`. `INV-LIM-1`: Konstanten in `contextra-types::limits`, Ports-Konstanten und `OpenConfig` werden aus diesem Register erzeugt (`tools/gen_limits.py`); der Test `limits_match_registry` bricht bei Abweichung.

## 1. Änderbarkeit

| Kürzel | Bedeutung |
|---|---|
| **K** | Konstante des Formats oder Codes; Änderung = Migration (SYS-2 §9) |
| **F** | Wird beim Anlegen der Instanz in `FORMAT` geschrieben und ist danach unveränderlich |
| **O** | `OpenConfig`, bei jedem Öffnen setzbar, wirkt bis zum Schließen |
| **R** | Zur Laufzeit änderbar (`Storage::tune`), nie persistiert, nie unter Formatwirkung |
| **P** | Pflicht, kein Standard; `open` ohne Wert ⇒ `InvalidInput` |

Ein Wert aus **F** wird einmal bei `Storage::create` gesetzt (nicht „nach dem ersten Cluster“; R14 ließ das offen, GAP-24). Der Wert `pack_bucket`/`pack_shards` ist damit ab Instanzerzeugung fest.

## 2. Register

### 2.1 Limits (`INV-LIM-1`, Prüfung vor jeder Sperre)

| Name | Standard | Bereich | Art | Wirkung bei Überschreitung |
|---|---:|---|---|---|
| `max_key_bytes` | 1 KiB | ≤ 4 KiB | je Space (K ≤ 4 KiB) | `Limit{KeyBytes}` |
| `max_header_bytes` | 4 KiB | ≤ 64 KiB | O | `Limit{HeaderBytes}` |
| `max_body_bytes` | 4 MiB | hart | K | `Limit{BodyBytes}` |
| `max_change_ops` | 4096 | hart | K | `ChangeTooLarge` |
| `max_change_bytes` | 8 MiB | hart | K | `ChangeTooLarge` |
| (Regel) `max_body_bytes ≤ max_change_bytes / 2` | | | K | beim Öffnen geprüft |
| `max_page_bytes` | 1 MiB | | O | `Limit{PageBytes}` |
| `max_clusters` | 16 Mio. | ≤ 64 Mio. | O (weich) | `Limit{Clusters}`; effektiv durch `cluster_meta_budget` |
| `max_branches_per_cluster` | 1024 | | K | `Limit{Branches}` |
| `max_branch_depth` | 4 | hart 8; > 4 nur mit `BranchOptions::deep_ack` | O | `Limit{BranchDepth}` |
| `max_merge_ops` | 4096 (= `max_change_ops`) | | K | `MergeTooLarge` |
| `max_gate_clusters` | 4096 | | K | `EraseFanoutExceeded` |
| `max_links` | 8 | Quellen je Link | K | `Limit{Links}` |
| `max_link_depth` | 8 | | K | `Limit{LinkDepth}` |
| `max_scope_ciphertext_bytes` | 256 KiB | | K | `Limit{Scope}` |
| `max_space_name_bytes` | 64 | | K | `InvalidInput` |

### 2.2 Platzierung und Formate

| Name | Standard | Art |
|---|---:|---|
| `block_target` | 16 KiB | F |
| `run_target_bytes` | 64 MiB | O |
| `blob_max_bytes` | 64 MiB | O |
| `redo_segment_bytes` | 64 MiB | O |
| `redo_mark_bytes` | 8 MiB | K |
| `redo_mark_interval` | 60 s (nur bei Schreiblast) | K (DEC-64) |
| `redo_max_age` | 1 h | O |
| `redo_align` | 4096 | K |
| `dir_page_bytes` | 4096 | K |
| `mini_section_max_bytes` | 4096 | K |
| `file_shard_count` | 1024 (3 Hex-Ziffern) | F |
| `pack_bucket` | `Quarter` | F (`Month`, `Quarter`, `Year`) |
| `pack_shards` | 8 | F (1…255) |
| `pure_threshold` | 8 MiB | je Space (≥ 1 MiB) |
| `pure_demote_ratio` | 0,25 | O |
| `max_pack_groups` | 4096 | O (je Space) |
| `spill_min_bytes` | 8 MiB | O |
| `spill_ratio` | 4 | O |
| `spill_max_age` | 6 h | O |
| `max_staging_runs` | 16 | O |
| `value_separation_min` | 1 KiB | je Space (256 B … `max_body_bytes`) |
| `catalog_block_bytes` | 1 MiB | K |
| `catalog_rollover_min_bytes` | 8 MiB | K (Schwelle: `max(catalog_rollover_min_bytes, Größe des letzten Snapshots)`) |
| `ledger_segment_bytes` | 16 MiB | K |
| `file_header_bytes` | 64 | K |
| `bloom_bits_per_key` | 10 | K |
| `cluster_filter_bits` | 8 | K |
| `rotation_recommended_seals` | 2^48 je (Schlüssel, Epoche) | K |

`pack_bucket = Quarter`, `pack_shards = 8` sind neue Standards (R14 nannte keinen). Begründung: bei Quartalsgranularität bleibt `max_pack_groups` (4096) für 500 Mandanten × 8 Shards × 1 offene Gruppe ausreichend; Jahresgranularität verlängert die Überaufbewahrung physischer Dateien nicht (Gate liegt am `due`, CYC-3 §11), vergröbert aber die Dateimischung.

### 2.3 Speicher

| Name | Standard | Art |
|---|---:|---|
| `memory_budget_bytes` | `min(1 GiB, max(256 MiB, 20 % des cgroup-/Job-Limits))`; ohne verlässliches Limit **P** | O/P |
| Anteile | siehe §3 | O |
| `memtable_bytes` | 64 MiB je Space (aktiv) | O |
| `max_immutable_memtables` | 3 je Space | O |
| `guard_window_entries` | `min(1 000 000, guard_budget / 24 B)` | O |
| `max_open_files` | 1024 | O |
| `fd_reserve` | 256 | O |
| `max_live_files` | 262 144 | O |
| `max_cluster_local_clusters` | 1024 (≤ 65 536) | O |
| `cluster_local_min_bytes` | = `pure_threshold` | abgeleitet |

### 2.4 Pipeline, Wartung, Platte

| Name | Standard | Art |
|---|---:|---|
| `commit_pipeline_depth` | 2 (Tiefe > 2 nur, wenn `FsCaps.overlapped_sync_beneficial`) | O |
| `max_group_changes` | 1024 | O |
| `max_group_bytes` | 8 MiB | O |
| `guard_retry_max` | 3 | K |
| `policy_retry_max` | 3 | K |
| `publish_timeout` | 1 s | O |
| `commit_wait_max` | 5 s (Wartezeit am Credit, danach `Backpressure(Retry)`) | O |
| `backpressure_soft` | 0,7 des Memtable-Gesamtbudgets (Verzögerung ≤ 1 ms je Commit, linear bis hart) | O |
| `backpressure_hard` | 0,9 (`Backpressure(Retry)`) | O |
| `reserve_stamps` | 2^20, Auffüllen bei < 50 % | K |
| `reserve_ids` | 2^16 | K |
| `reserve_space_ids` | 2^16 | K |
| `reserve_stamps_urgent` | 2^10 | K |
| `reserve_ids_urgent` | 2^8 | K |
| `maint_workers` | `max(1, min(cpus, 4))` | O |
| `maint_io_fraction` | 0,5 | R |
| `max_lag_bytes` | 1 GiB | O |
| `l0_compaction_trigger` | 4 Runs je Wartungsgruppe | O |
| `level_multiplier` | 10 | O |
| `max_levels` | 7 | K |
| `compaction_job_max_bytes` | 10 × `run_target_bytes` | O |
| `blob_gc_ratio` | 0,5 Lochanteil | O |
| `blob_gc_min_bytes` | 8 MiB | O |
| `scrub_interval` | 7 d für einen Vollzyklus | O |
| `scrub_io_fraction` | 0,1 der Wartungsbandbreite | R |
| `diskfull_enter_bytes` | `max(512 MiB, 2 % des Volumens)` freier Platz | O |
| `diskfull_exit_bytes` | 2 × `diskfull_enter_bytes` (Hysterese) | O |
| `erase_reserve_bytes` | ≥ 2 GiB; **enthält** Scratch, Notsegmente (Redo, Catalog, Ledger) und Dateisystem-Headroom | O |
| `purge_scratch_reserve_bytes` | ≥ 2 × max(`run_target_bytes`, `blob_max_bytes`, `redo_segment_bytes`) = 128 MiB | O |
| `anchor_max_age` | 1 s | O |

### 2.5 Löschung, Schlüssel, Zeit

| Name | Standard | Art |
|---|---:|---|
| `purge_deadline` | **P** (je Auftrag) | P |
| `purge_deadline_min` | 6 h | K |
| `purge_window_fraction` | 0,3 (Fenster `W = 0,3 × D`) | O |
| `deadline_reserve_fraction` | 0,1 | O |
| `key_destroy_fraction_max` | 0,6 (`max_delay ≤ 0,6 × D`, GAP-22) | abgeleitet: `1 − W − Reserve` |
| `purge_trigger_fraction` | 0,6 (Fristanteil, ab dem Purge zwingend startet; ersetzt „70 %“ und „60 %“ aus R14) | O |
| `sealed_purge_escalate_flush` | 0,6 | O |
| `sealed_purge_escalate_urgent` | 0,8 | O |
| `purge_batch_min` | 16 Cluster je Datei | O |
| `purge_rewrite_concurrency` | 1 | O |
| `link_fence_ttl` | 5 s | O |
| `logical_only_alarm_age` | 30 d | O |
| `orphan_key_grace` | 24 h | O |
| `key_grace` | 10 min | O |
| `revoke_grace` | 5 s | O |
| `view_lease` | 60 s | O |
| `view_lease_max` | 10 min | O |
| `max_hold_ttl` | 24 h (= `idem_ttl`) | K |
| `idem_ttl` | 24 h | O |
| `idem_max_active` | 1 048 576 Tokens | O |
| `idem_max_bytes` | 512 MiB | O |
| `max_clock_skew` | 5 min | O |
| `history_retention` | 7 d (nur `KeepUntilFloor`) | O |
| `tomb_epoch_rotation` | aus | O |
| `backup_horizon` | **P**, sobald Retire aktiv | P |
| `max_active_tomb_epochs` | 4 | K |
| `LiftPolicy` | `OwnerOrAdmin` | O |
| `max_retention_extension_restrictions` | unbegrenzt (nur Observer-Zähler) | — |

### 2.6 Retrieval und Profil (Engine; hier nur Namen und Werte)

| Name | Standard |
|---|---:|
| `micro_index_max_vectors` | 50 000 |
| `cluster_index_max_vectors` | 2 Mio. |
| `emb_precision` | `F16` (`F32`, `Sq8` = Profil `Lean`) |
| `max_partition_vectors` / `_postings_bytes` / `_edges` (`Corpus`) | 500 000 / 256 MiB / 5 Mio. |
| `…_personal` (`Pseudo`) | 100 000 / 64 MiB / 1 Mio. |
| `max_graph_segments_per_tenant` | 64 |
| `rerank_depth` | 4 × k |
| `ryw_wait` | 50 ms |
| `staged_orphan_ttl` | 24 h |
| `k_min` | 50 |
| `restore_slack_fraction` | 0,05 |
| `scan_fallback_max_vectors` | 1 Mio. |
| `export_io_fraction` | 0,2 |
| `filter_selectivity` | 0,05 |
| `ComplianceProfile` | `Standard` |
| `pseudonym_scope` | `PerPerson` |
| `placeholder_retention` | `Keep` (`Strict`: `Scrub`) |
| `snapshot_set_lease_max` | 24 h |

## 3. Speicherbudget

`memory_budget_bytes = B`. Jede Allokation ist `try_reserve(ResourceClass, bytes)`; das Budget hält je Klasse eine Obergrenze (`Limit{Class}`), keine Unterschreitungsgarantie.

| `ResourceClass` | Anteil an B | Bei B = 1 GiB |
|---|---:|---:|
| `Memtable` | 25 % (Obergrenze 256 MiB) | 256 MiB |
| `ClusterMeta` / `ClusterHot` | 15 % (`cluster_meta_budget`) | 161 061 273 B |
| `GuardWindow` | 2 % | ≈ 21,5 MB |
| `BlockCache` + `DirIndex` | 20 % | ≈ 205 MiB |
| `KeyCache` | 1 % | ≈ 10,7 MiB |
| `Extension` (davon `IndexRebuild` 10 % von B) | 20 % | ≈ 205 MiB |
| `Inflight` (Changes, Rewrite-Puffer) | 10 % | ≈ 107 MiB |
| `Headroom` (Allocator, Runtime) | 7 % | ≈ 75 MiB |
| Summe | 100 % | |

Rechnungen (geprüft):

1. `resident_clusters_max = cluster_meta_budget / 40 B`. Bei 1 GiB: `161 061 273 / 40 = 4 026 531` (R14 nannte 3,9 Mio.; GAP-23).
2. 16 Mio. Cluster × 40 B = 640 000 000 B ≈ 610 MiB, das entspricht B ≈ 4,07 GiB.
3. Guard-Fenstereintrag = 24 B (`stamp u64`, `fp u64`, `ck_hash u64`); bei 2 % von 1 GiB ⇒ ≈ 894 000 Einträge, also `guard_window_entries = 894 000` (< 1 Mio.).
4. **Skalierung bei kleinem B** (Entwicklungsrechner, 256 MiB ≤ B): `memtable_total = min(256 MiB, 0,25 B)`, `memtable_bytes_effective = min([memtable_bytes], memtable_total / (1 + [max_immutable_memtables]))`. Bei B = 256 MiB ergibt das 64 MiB gesamt und 16 MiB je Space-Memtable. Das Gesamtbudget für Memtables ist maßgeblich, nicht die Spaceanzahl.
5. Auf der Zielmaschine mit 3,9 GiB RAM: B = `min(1 GiB, 0,2 × 3,9 GiB = 798 MiB)` = 798 MiB (cgroup-Limit fehlt ⇒ Wert Pflicht, Entwicklungsprofil setzt 512 MiB).

Unter Druck gilt die Reihenfolge der Freigabe: `BlockCache` → `Extension` (Indexaufbau drosseln) → `Memtable` (früher flushen) → erst dann `Backpressure`. `Inflight` für Gate/Control/Link und `KeyCache` für Löschpfade werden nie verdrängt (`INV-DISK-1`-Linie auch für RAM: Löschung bleibt ausführbar).

## 4. Admission (`INV-ADM-1`)

Jede Admission wirkt vor der Allokation und liefert eine eigene Fehlerklasse; keine stille Herabstufung.

| Struktur | Residenz | Kosten | Klasse |
|---|---|---|---|
| `ClusterIndex` | immer | 36 B (40 B mit Slack) | `ClusterMeta` |
| `HotRecord` (Branch-Tabelle, Restrictions, Links, Pins, Views, Cursor) | bei Bedarf; nicht kaltstellbar bei Restriction/Link/Pin | ≤ 256 B + Inhalt | `ClusterHot` |
| Directory-Sparse-Index + `ClusterFilter` | je Datei | ≈ 1,3 B je Cluster und Datei | `DirIndex` |

1. `register` reserviert `ClusterMeta`; Fehlschlag ⇒ `Limit{ClusterMetaBudget}`, kein Teilzustand. `open` mit mehr Clustern als `resident_clusters_max` ⇒ `ReadOnly` mit Diagnose.
2. `ClusterLocal` nur wenn `expected_bytes ≥ [cluster_local_min_bytes]`, Zahl lebender `ClusterLocal`-Cluster `< [max_cluster_local_clusters]` und `[max_live_files]` Reserve für zwei Dateien je Level; sonst `Limit{ClusterLocalAdmission}`. Massen personenbezogener Cluster nutzen `PackedBounded` plus Crypto-Erase.
3. Pure-Demotion (R-PL-3): nicht-`ClusterLocal`-Cluster unter `[pure_demote_ratio] × [pure_threshold]` gehen per `rebucket` in ihre Gruppe zurück.
4. `max_live_files`: ab 80 % hebt der Governor `pure_threshold` je Stufe ×2 (nur Laufzeit, nicht persistiert); bei 100 % entstehen keine neuen Pure-Dateien. Gate, KeyDestroy, Purge bleiben ausführbar.
5. FD-Cache: `[max_open_files]` ist LRU-Größe, kein Dateilimit; Lane `urgent` hat `[fd_reserve]`. `open` prüft `RLIMIT_NOFILE ≥ [max_open_files] + [fd_reserve]`, sonst `FdLimitTooLow`.
6. Packgruppen: `max_pack_groups` je Space; eine Registrierung, die eine neue Gruppe über dem Limit anlegen würde, ⇒ `Limit{PackGroups}` (Abhilfe: Limit erhöhen, RAM O(Gruppen), oder `pack_bucket`/`pack_shards` bei der nächsten Instanz grober wählen; Instanz-Parameter sind nach `create` fest).
7. Idempotenz: Zahl aktiver Tokens `> [idem_max_active]` oder Bytes `> [idem_max_bytes]` ⇒ `IdemCapacity` vor jeder Mutation; aktive Tokens werden nie verdrängt.
8. **Konvention Session ≠ Cluster** (CYC-7 §10): Cluster folgt der Löschdomäne. Die Fassade meldet `ClusterMetaBudget`-Druck ab 70 % mit `hint()`.

## 5. Validierung von `OpenConfig`

Reihenfolge (erster Fehler gewinnt, alles vor jeder E/A auf Daten):

1. Typ und Bereich jedes Werts gegen §2 (`InvalidInput{field}`).
2. Querregeln: `max_body_bytes ≤ max_change_bytes/2`; `purge_deadline_min ≥ 6 h`; `purge_window_fraction + deadline_reserve_fraction + key_destroy_fraction_max = 1`; `diskfull_exit_bytes ≥ 2 × diskfull_enter_bytes`; `erase_reserve_bytes ≥ purge_scratch_reserve_bytes + 3 × [ledger_segment_bytes] + [redo_segment_bytes] + [catalog_block_bytes]`.
3. Ports vollständig (`Ports::validate`): fehlende Pflichtports ⇒ `InvalidInput{port}`; `MemFs`/`FaultFs`/`Sim*` nur mit Feature `testing` (DEC-43).
4. Profilprüfung (`ComplianceProfile`, SYS-5 §4) ⇒ `ProfileViolation{rule}`.
5. Umgebung: `RLIMIT_NOFILE`, `free_space ≥ erase_reserve_bytes + diskfull_enter_bytes`, Dateisystem-Fähigkeiten (`FsCaps.prealloc_reserves_space` fehlt ⇒ harter DiskFull-Erase-Modus verweigert oder Restrisiko im Receipt), `HostProbe`.
6. Danach erst `FORMAT`/`LOCK`/Recovery (CYC-6 §3).

## 6. Fristarithmetik

`D = purge_deadline`, `W = 0,3 D`, `R = 0,1 D`. Zulässig nur, wenn `max_delay(KeyProvider) ≤ 0,6 D` (`INV-DL-1`); sonst `DeadlineBeforeKeyDestroy`. Beispiele: `D = 6 h` ⇒ `max_delay ≤ 3,6 h`; `D = 72 h` ⇒ ≤ 43,2 h. Auslöser (das Erste, was eintritt): natürliche Compaction/Blob-GC, `0,6 D` verbraucht (`purge_trigger_fraction`), `purge_batch_min` Cluster in derselben Datei. Eskalation Sealed: Lane `compact` → `flush` bei 0,6 D → `urgent` bei 0,8 D. Index-Rebuild verwendet dieselben Schwellen (CYC-7 §5).

## 7. Metrikkatalog (Z-16)

Ein fester Katalog `metrics.rs` definiert Name, Typ (`counter`, `gauge`, `histogram`), Einheit, erlaubte Labels. Labelwerte stammen nur aus den geschlossenen Enums `durability{durable,relaxed}`, `result{ok,err}`, `file_role` (SYS-1 §3), `resource_class` (§3), `phase`, `stage` (CYC-3 §2), `owner_kind`, `error_class` (SYS-1 §6). **Verboten als Label oder Observer-Payload:** Tenant-, Cluster-, Space-Kennung, Schlüssel, Token, Querytexte, Inhalte, Embeddings, Dateipfade.

| Familie | Metriken (Name · Typ · Labels) |
|---|---|
| Commit | `commits_total` counter {durability,result} · `commit_group_size` histogram · `commit_latency_seconds` histogram {durability} · `commit_credit_wait_seconds` histogram |
| Durability | `stamp_lag` gauge (`visible − durable`, Epochen) · `fsync_latency_seconds` histogram {file_role} · `redo_bytes_total` counter |
| Ressourcen | `resource_used_bytes` / `resource_limit_bytes` gauge {resource_class} · `block_cache_hit_ratio` gauge · `erase_reserve_free_bytes` gauge · `scratch_free_bytes` gauge · `fds_open` gauge |
| Wartung | `maint_bytes_total` counter {phase} · `maint_seconds` histogram {phase} · `write_amplification` gauge {phase} · `l0_files` gauge · `staging_runs` gauge |
| Löschung | `erase_tickets_open` gauge · `erase_oldest_ticket_age_seconds` gauge · `erase_deadline_breaches_total` counter · `erase_stage_seconds` histogram {stage} · `gate_latency_seconds` histogram · `key_destroy_latency_seconds` histogram · `purge_bytes_total` counter · `anchor_lag_seconds` gauge · `logical_only_clusters_aged` gauge · `poisoned_erase_pending` gauge |
| Extensions | `owner_watermark_lag` gauge {owner_kind} · `owner_blocked` gauge {owner_kind} · `index_rebuild_bytes_total` counter |
| Guards | `guard_conflicts_total` counter (Stufe A) · `guard_retry_total` counter (Stufe B) · `guard_window_floor_hits_total` counter |
| Branches | `branches_live` gauge · `branch_depth_max` gauge · `oldest_pin_age_seconds` gauge |
| Fehler | `errors_total` counter {error_class} · `poisoned_handles` gauge · `leases_expired_total` counter |
| Schlüssel/Tomb | `tomb_epochs_open` gauge · `tomb_epochs_closed` gauge · `rotation_recommended_total` counter · `key_provider_latency_seconds` histogram |

`Metrics`/`Observer` sind best-effort, nicht blockierend, außerhalb aller Kernsperren; ihr Ausfall ändert keine Semantik. Der Inspector liefert standardmäßig aggregierte oder opake Werte, braucht eine Capability und entsiegelt nie. Release-Gates: Katalog-Review und Kardinalitäts-/Leakage-Test (`metrics_cardinality`, `metrics_no_ids`).

## 8. Entscheidungen und geschlossene Lücken

DEC-41 (ein Zahlenregister), DEC-42 (Änderbarkeitskürzel K/F/O/R/P), DEC-44 (Standard `Quarter`/8 für Packgruppen), DEC-45 (Budgetskalierung bei kleinem B); GAP-21, GAP-22, GAP-23, GAP-24, GAP-25, GAP-26 (SYS-7 §3).

## 9. Abnahme

`limits_match_registry`, `open_config_validation` (jeder Fehlerpfad §5 einzeln), `budget_arithmetic` (§3 Nr. 1–4 als Property), `admission_*` (je Struktur), `metrics_cardinality`, `metrics_no_ids`, `deadline_arithmetic` (§6, Randwerte).

**Offene Punkte:** keine.
