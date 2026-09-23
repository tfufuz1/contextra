---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "16"
---
## 16. Vollständige Abnahmekriterien

### 16.1 Kernsystem-Abnahmekriterien

| Nr. | Kriterium | Reifegrad |
|---|---|---|
| K-1 | `insert_lock` vollständig durch key-granulare `kv_locks` ersetzt | 🟢 |
| K-2 | HNSW-Hotpath nutzt unaligned-SIMD-Distanzkernel statt Heap-Allokation | 🟢 |
| K-3 | BM25 nutzt residenten Postinglisten-Index mit Block-Max WAND | 🟢 |
| K-4 | Block-Cache erzwingt bei Lesetreffer im Opt-in-Backend keinen Write-Lock | 🟡 |
| K-5 | DiskANN löscht ohne HNSW-Fallback (native Tombstones) | 🟢 |
| K-6 | Bandit-Latenzbudget CI-gated | 🟢 |
| K-7 | `compact()` blockiert keine nebenläufigen Leser (RCU-Swap) | 🟢 |
| K-8 | PPR proportional zur Seed-Menge, nicht zur Graphgröße (P24) | 🟢 |
| K-9 | `build_provenance` nutzt `ProvenanceBuilder`-Struct | 🟢 |
| K-10 | FlatBuffers-Drift-Gate aktiv | 🟢 |
| K-11 | `SignalKind::from_name` allokationsfrei (`eq_ignore_ascii_case`) | 🟢 |
| K-12 | BM25F produktiv nutzbar | 🟢 |
| K-13 | KV-Cache-Bridge LSM-Fallback mit Testabdeckung für alle Kombinationen | 🟢 |
| K-14 | Drift-Alpha gedeckelt | 🟢 |
| K-15 | Cloud-Egress 5-Schichten produktiv, Rehydration inkl. UTF-8-Sicherheit | 🟢 |
| K-16 | ⚖️ Bandit-Default → ShermanMorrison (sobald Gate besteht) | ⚖️ |
| K-17 | ⚖️ Block-Cache-Default → SIEVE (sobald entschieden) | ⚖️ |
| K-18 | ⚖️ DocId-128 als Produktions-Default (Major-Release) | ⚖️ |
| K-19 | Group-Commit-Loom-Test sichtbar grün in CI | 🔴 |

### 16.2 Abnahmekriterien AK-1 bis AK-15 (normativ und abschließend; AK-9 bis AK-15 ab Fassung 2.1)

| AK | Kriterium | Nachweis (Testpfad) |
|---|---|---|
| AK-1 | `HyperEdge` mit ≥3 `RoleBinding`s persistiert, restart-fest, per `hyperedges_for_entity` auffindbar, konsistent unter gleichzeitigem `compact()` | `hyperedge_persistence_survives_restart.rs`, `hyperedge_compact_race.rs` |
| AK-2 | `estimate_memory_bytes()` inkl. Hyperkanten und capacity-basiert; `estimate_compaction_peak_bytes()` unterschätzt den mit Zählallokator gemessenen Spitzenwert nie und überschätzt ihn höchstens um Faktor 1,5; `compact_async`-Budget-Check greift gegen die Spitze | `hyperedge_memory_budget.rs` |
| AK-3 | Zwei gleichzeitige `relate_n_ary` mit überlappenden, unterschiedlich geordneten Mengen deadlockfrei | `loom_relate_n_ary.rs` |
| AK-4 | `SignalKind` strukturell unverändert (kein `Hyperedge`-Signal) | `signal_kind_no_new_variant.rs` |
| AK-5 | FlatBuffers-Drift-Gate grün **vor** `HyperEdgeFb`-Merge | CI-Job-Abhängigkeit `hyperedge-schema-merge: needs: [flatbuffers-drift-gate]` |
| AK-6 | Cascade bricht bei >1.000 Hyperkanten kontrolliert auf Hintergrundverarbeitung um | `hyperedge_cascade_fanout.rs` |
| AK-7 | `hyperedges_included` im Report sichtbar `false` ohne Projektion | `community_hyperedges_included_flag.rs` |
| AK-8 | Keine Regression auf binäre `relate()`/`Edge`-Benchmarks | `binary_edge_regression.rs` |
| AK-9 | `as_view()` und Snapshot-Klon kopieren keine Teilnehmer: `Arc::ptr_eq` auf `participants`, Allokationsvolumen des Klons unabhängig von `Σ\|e\|` | `hyperedge_payload_sharing.rs` |
| AK-10 | Stern-Expansion entspricht der konfigurierten Clique-Konvention (Schur-Komplement), N=2 ≙ binäre Kante, deterministische Reihenfolge, kein Klon der Hyperkanten | `star_expansion_equals_clique.rs` |
| AK-11 | Cascade überlebt Absturz nach Commit: alle Hyperkanten genau einmal tombstoniert, Queue leer, Wiederholung ist No-op; `DeletionProof` nur bei vollständiger Löschung | `hyperedge_cascade_crash_recovery.rs` |
| AK-12 | SQ8-Bias wird gemessen und im Header gebunden an die Codebook-Version; Genauigkeitsgrenzen laut §7.4 | `sq8_bias_calibration.rs` |
| AK-13 | WAL-Queue ist begrenzt: `Backpressure`/Warten bei voller Queue, `append` kehrt erst nach `fsync` zurück | `wal_backpressure.rs` |
| AK-14 | Shard-Zuordnung ist je `KvKeyLocks`-Instanz stabil; gleiche Entität ⇒ gleicher Shard | `kv_locks_stable_shard.rs` |
| AK-15 | Jeder Routing-Datensatz trägt `propensity ≥ 0,01` (randomisierte Logging-Policy) | `ips_requires_propensity.rs` |
| AK-16 *(Fassung 4, §21.1)* | `thresholded_local_hfd` läuft im selben `ShadowMode`-Pfad wie `forward_push_ppr` und `DensePowerIteration`; Diskrepanz-Logging über dieselbe Mechanik wie `PprAlgorithm::ShadowMode`; Top-k-Tie-Breaker nutzt totale `EntityId`-Ordnung | `tlhfd_shadow_mode_parity.rs` |
| AK-17 *(Fassung 4, §21.2)* | DiBud wird **nicht** gegen die heutige `fuse_signals(Vec<...>, usize)`-Signatur gemerged; ein PR, der `DiBudFusionState` einführt, ohne zuvor `Iterator<Item = DocId>`-Grenzen in `contextra-vector` und `contextra-text` bereitzustellen, schlägt das Gate fehl | `dibud_requires_streaming_channels.rs` |
| AK-18 *(Fassung 4, §21.3)* | `update_with_flow` aktualisiert `drift_rate` (Transport-Vektor $\hat\delta_t$) ausschließlich aus einem Ring-3-Kontext; ein Aufruf aus einem synchronen Ring-0-Hot-Path-Test schlägt fehl (Architektur-Lint, kein Laufzeit-Panic) | `fcts_drift_update_ring3_only.rs` |
| AK-19 *(Fassung 4, §21.4)* | `cascade_invalidate_hyperedges_for_superseded_doc` traversiert `child_edge_ids` rekursiv bis `MAX_HYPEREDGE_CASCADE_FANOUT`; ein gelöschtes Quelldokument, dessen Fakten in einen LeanRAG-Super-Knoten abstrahiert wurden, ist nach Cascade-Lauf nicht mehr auffindbar; FlatBuffers-Drift-Gate für `child_edge_ids` grün **vor** Merge (H4) | `leanrag_cascade_through_superedge.rs`, CI-Job-Abhängigkeit `leanrag-schema-merge: needs: [flatbuffers-drift-gate]` |

---

<a id="17-optimierungen"></a>
