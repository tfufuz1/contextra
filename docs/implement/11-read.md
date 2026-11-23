# SP-READ — Views, Cursor, Page, Cache

| | |
|---|---|
| Modul | `engine/{views,cursor,page,cache}.rs` |
| Besitzt | Snapshot-Sichten, Punkt-/Bereichslesen, Pagination, Gewichtsbudget, Cache-Politik |
| Konsumiert | PORTS, MEMVER, RUN, BLOB, CLUSTER, BRANCH |
| Spec-Bezug | R8 §7, §8, §12.4; INV-VIEW-1/2, INV-VIS-1, INV-CONS-1 |
| Phase | P1 Basis; P4 vollständiger Lesepfad (G-R) |

## 1 Auftrag
Eine View bindet unveränderliche Version und sichtbaren Stamp konsistent. `view_durable` sowie Consumer-/Page-Reads begrenzen Ergebnisse auf `durable_stamp`. Core liefert sortierte Köpfe und Gewichte; Ranking-/Retrieval-Politik bleibt oben.

## 2 Verträge
`Storage::view/view_at/view_durable`; `View::get`, `scan`, `page`. `page` budgetiert anhand `weights[4]` und `weight_sum`; bei Sealed-Clustern entschlüsselt es kleine SubIndex-Metadaten, aber keine Body-Blöcke für Skip-Entscheidungen. Cursor/ResumeToken sind an View/Stamp gebunden und versiegelt, sofern sie persistiert oder weitergereicht werden. Alle Extensions werden vor Rückgabe durch `filter_visible` geführt.

## 3 Regeln
Branch-Kappen werden an RUN weitergegeben. Tombstone und RangeDelete werden vor Ergebnisbildung angewendet. `page` lädt Köpfe ohne Körper, sofern nicht explizit verlangt. Cache-Einträge sind versionsbezogen oder invalidiert; ein Gate kann keine bereits veröffentlichte View logisch zurückholen, aber alle Ergebnisse mit Stamp ≥ S als Erased ausweisen.

## 4 Invarianten / Umsetzung
INV-VIEW-1/2, INV-VIS-1, INV-CONS-1. Tests: Snapshot bleibt stabil bei parallelem Commit; `view_durable` überschreitet nie Watermark; Extension-Test erzwingt `filter_visible`; Page-Ergebnis entspricht Vollscan und hält Budget.

## 5 Arbeitspakete
WP-11.1 View-Snapshot und Punktlese (P1) · WP-11.2 Iterator/Merge + Tombstone (P4) · WP-11.3 Page/Weight Budget (P4) · WP-11.4 Cursor/ResumeToken (P4) · WP-11.5 Cache/Invalidierung (P4). Abnahme: Referenzmodellgleichheit, keine Sichtbarkeitslecks.
