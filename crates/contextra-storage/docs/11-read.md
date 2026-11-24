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

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §7, §11.4 (View-Lease), §12.2 (BlockPruner-Aufruf); `INV-VIEW-3`, `INV-PRUNE-1`; E-76, E-79.

- **View-Lease (E-76):** `View`/`Cursor` tragen `lease_deadline` (monotone `Clock`, Standard `view_lease` 60 s, `renew()` bis `view_lease_max` 10 min). Ablauf setzt dieselbe Widerrufsmechanik wie `ViewPolicy::Revoke` (Referenz lösen, Graveyard freigeben); jede weitere Nutzung (`get`, `body`, `page`, `poll`, `filter_visible`) liefert `SnapshotTooOld{stamp, floor_hint}`. Cursor prüfen die Lease zwischen Blöcken zusammen mit Gate und Widerruf. `SnapshotTooOld` ≠ `BelowFloor`. `ViewPolicy::Wait(timeout)` wartet höchstens bis `min(timeout, Lease-Ende)`. Zeit nie aus UTC.
- **BlockPruner (K-29):** `page`/`Cursor` akzeptieren einen optionalen `&dyn BlockPruner`, rufen ihn nur für `ShadowSafe`-Besitzer und nur auf `branch == 0` auf und lassen Gate/Restriction/`filter_visible` unabhängig davon laufen.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-11.6 | View-Lease, `renew`, `SnapshotTooOld`; Pruner-Hook in `page`/`Cursor` | 1.5 | WP-11.4, WP-01.5 | P4 | Lease-Ablauf-Test mit Sim-Clock, Graveyard-Freigabe nach Ablauf, Cursor-Abbruch zwischen Blöcken, Property „mit Pruner ≡ ohne Pruner“ (`INV-PRUNE-1`), `INV-VIEW-3` |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

Lesen prüft Gate/Generation vor Payload und erneut vor Ausgabe. Nach einem Gate ist `Shredded` nicht beobachtbar: Ergebnis ist `Erased`, `KeyUnavailable` oder `KeyLost`. Branch-Resolve nutzt den branch-bewussten Run-Index; View-Lease-Ablauf bleibt von `BelowFloor` getrennt und ergibt `SnapshotTooOld`.
