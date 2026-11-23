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
`SegmentOwner`: `prepare`, `on_gate`, `verify_absent`, `rewrite_without`; Vertrag deklariert `ClusterSections | MixedPerClusterKey | Rebuildable{max_age} | NoPersonalData`, Key-Scope, Watermark, Rewrite-/Rebuild-Obergrenze, RAM/Scratch-Budget und attestierbare Evidence. `ExtensionBuilder` schreibt opaque Blocks; `ConsumerFeed` liest Redo mit `durable_stamp`, `replay_floor` und Consumer-eigenem Floor. ChangeFeed unterscheidet `Redacted` ab Gate und `Erased` nach Purge. Consumer/Indexer dürfen nie Stamps > durable bestätigen.

## 3 Regeln
`applied_commit_id` ist Index-Watermark und darf keine Lücke behaupten; er darf nur bis zum höchsten angewandten `durable_stamp` reichen. Query verwendet Basisindex plus vollständiges Delta bis V oder liefert explizit `IndexLag`. Kandidaten werden nach Suche und erneut vor Rückgabe gegen Gate-Generation gefiltert. Globaler Segment-Key ist für personenbezogene `Sealed`-Daten unzulässig. `Rebuildable` bleibt `Purging` bis attestierter Rebuild/Verify. Extension ohne erfolgreichen `verify_absent` blockiert erforderliche Erase-Stufe. Aufruf außerhalb Sperren. Opaque Formatversion unbekannt ⇒ Unsupported.

## 4 Invarianten / Umsetzung
INV-VIS-1, INV-CONS-1, INV-ERA-2/6/7/8. Contract tests für verspätete/fehlerhafte Extension, Gate während Build/Query, Consumer-Neustart, Watermark-Lücken, fehlenden Key-Attest, `Rebuildable`-Deadline und Resource-Budget-Erschöpfung.

## 5 Arbeitspakete
WP-17.1 durable Feed/Watermarks (P5) · WP-17.2 ExtensionBlock IO/Manifestklassen (G-F/P6) · WP-17.3 SegmentOwner Key-Scope/Gate/Verify/Evidence (P6) · WP-17.4 Consumer retention floors + Lag-Epoch (P6) · WP-17.5 Gesamtsystem-Ressourcenbudget (G-0 bis G-P). Abnahme: kein Erweiterungsergebnis leakt gated Daten oder bestätigt nichtdurable Stamps; kein Complete ohne attestierte Löschklasse.
