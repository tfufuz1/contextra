# SP-EXT — Extension-Segmente, Change-Feed, Consumer

| | |
|---|---|
| Modul | `engine/{segments,extension,feed}.rs` |
| Besitzt | fünf Erweiterungsverträge, Extension-Blocks, durable Consumer-Offsets, Watermarks |
| Konsumiert | PORTS, READ, COMMIT, CLUSTER, ERASE, CATALOG |
| Spec-Bezug | R8 §12; INV-VIS-1, INV-CONS-1, INV-ERA-2/6 |
| Phase | P5 Feed; P6 Erase-Integration (G-E) |

## 1 Auftrag
Kern verwaltet keine ANN/FTS/Graph-Algorithmen. Erweiterungen liefern abgeleitete Segmente mit Eigentümervertrag, expliziter Schutzklasse, Ressourcenbudget und Watermark. Alle Extension-Ergebnisse durchlaufen verpflichtend die Query-Fassade mit `View::filter_visible`; Erase-Rückrufe finden ohne Kernlocks statt.

## 2 Verträge
`SegmentOwner`: Manifest mit `ClusterSections | MixedPerClusterKey | MixedPerSubjectKey | Rebuildable{max_age,max_rewrite_bytes} | NoPersonalData`, Key-Scope, Lineage, Watermark, Rewrite-Obergrenze und RAM/Scratch-Budget; Methoden `on_gate`, `rewrite_without`, `verify_absent`. `MixedPerSubjectKey` ist nur für per-record KMS-EntryDEKs zulässig; `NoPersonalData` erfordert versionierte Provenienz-Evidence. `EraseAttestation{owner,generation,scope_digest,applied_position:(InstanceId,Stamp),keys_destroyed,segments_rewritten,evidence_digest}` ist notwendig, aber ein Rewrite-Ergebnis allein genügt nicht. `ExtensionBuilder` schreibt opaque Blocks; `ConsumerFeed` liest Redo mit `(instance_id,durable_stamp)`, `replay_floor` und Consumer-eigenem Floor. ChangeFeed unterscheidet `Redacted` ab Gate und `Erased` nach Purge. Consumer/Indexer dürfen nie Stamps > durable bestätigen. Indexer binden jede Rebuild-/Query-Generation an `view_durable()`; `view()` auf Relaxed-Sicht ist als Indexquelle verboten. Ingest schreibt kanonische Dokument-, Vektor-, Text-/Graph-Quelldaten atomar in einem `Change`, soweit sie dieselbe Cluster-Löschdomäne teilen; externe HNSW/BM25/CSR-Strukturen bleiben abgeleitete, nicht atomare Generationen. Query prüft Tenant-/Rechtefilter vor Indexexploration und Gate + `filter_visible` vor Scoring/Traversal sowie vor Rückgabe. Watermark-Lücken müssen durch Delta/Fallback geschlossen oder als `IndexLag` gemeldet werden.

## 3 Regeln
`applied_position=(instance_id,stamp)` ist ein lückenloser Feed-Watermark und darf nur bis zum höchsten vollständig angewandten `durable_stamp` derselben Instanz reichen. Query verwendet Basisindex plus vollständiges Delta bis V oder liefert explizit `IndexLag`. Gate-Filter läuft vor Scoring, Graph-Expansion und Payload-Auswertung und erneut unmittelbar vor Rückgabe mit Generation-Check. Globaler Segment-Key ist für personenbezogene `Sealed`-Daten unzulässig. `MixedPerClusterKey` gilt nur, wenn Payload **und alle Kanten/Posting-/Routing-Metadaten** ausschließlich einem Cluster angehören. Clusterübergreifende HNSW-/CSR-Kanten, BM25-Postings oder Centroids sind `Rebuildable` mit hartem `max_rewrite_bytes`/`max_age`; das ist kein per-Cluster Crypto-Erase und keine O(cluster)-Purge-Zusage. Ohne pro-Cluster-Schlüssel über alle persistenten Kopien ist fehlender Owner-/Scope-Nachweis `Incomplete` (kein erfolgreicher Löschbeleg); `CompleteWithResidual` ist nur nach Abschluss mit ausdrücklich außerhalb des Vertrags liegender, benannter Restexposition zulässig. `on_gate` ist Invalidierungs-/Wakeup-Hinweis, keine Korrektheitsquelle. `Rebuildable` bleibt `Purging` bis attestierter Rebuild/Verify. Extension ohne gültige `EraseAttestation` blockiert erforderliche Erase-Stufe. Jede Builder-/Query-/Rewrite-Allokation reserviert vorher RAM/Scratch über `MemoryBudget`; Aufruf außerhalb Sperren. Opaque Formatversion unbekannt ⇒ Unsupported.

## 4 Invarianten / Umsetzung
INV-VIS-1, INV-CONS-1, INV-ERA-2/6/7/8. Contract tests für verspätete/fehlerhafte Extension, Gate während Build/Query, Consumer-Neustart, Watermark-Lücken, fehlenden Key-Attest, `Rebuildable`-Deadline und Resource-Budget-Erschöpfung.

## 5 Arbeitspakete
WP-17.1 durable Feed/Watermarks (P5) · WP-17.2 ExtensionBlock IO/Manifestklassen (G-F/P6) · WP-17.3 SegmentOwner Key-Scope/Gate/Verify/Evidence (P6) · WP-17.4 Consumer retention floors + Lag-Epoch (P6) · WP-17.5 Gesamtsystem-Ressourcenbudget (G-0 bis G-P). Abnahme: kein Erweiterungsergebnis leakt gated Daten oder bestätigt nichtdurable Stamps; kein Complete ohne attestierte Löschklasse.
