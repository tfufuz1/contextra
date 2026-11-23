# Formale Modelle für R8

Diese Modelle sind unabhängige Spezifikationen für Sicherheits- und Liveness-Eigenschaften. Sie implementieren keine Produktionslogik. Die normative Semantik steht in `crates/contextra-storage/SPEC.md`; bei Modellkonflikten wird zuerst die SPEC geklärt und erst danach das Modell angepasst. Ein grüner Modelllauf ersetzt weder FaultFs- noch Integrationstests.

## Konventionen

- TLA+ Modelle verwenden endliche, kleine Konstanten (`Clusters`, `Branches`, `Stamps`, `Keys`, `Files`, `Owners`) und explizite Typinvarianten. Keine Modellvariable repräsentiert unbeschränkte Bytepuffer.
- Jeder Zustandsübergang benennt die persistente Grenze, die er repräsentiert. Ein Crash verwirft volatile Variablen und erhält ausschließlich die als durable markierten Präfixe.
- Liveness-Eigenschaften nennen Fairnessannahmen explizit. KMS, Filesystem, Anchor oder Extension dürfen ohne angenommene Antwort keine Fortschrittsgarantie erhalten.
- Jede Eigenschaft referenziert die betroffenen `INV-*` und Audit-IDs. Modellvereinfachungen und ausgeschlossene Zustände stehen im Modellkopf.
- Modellbeispiele verwenden keine produktiven IDs, Schlüssel oder Nutzdaten.

## Modellinventar und Gates

| Modell | Mindestzustand / Aktionen | zentrale Sicherheitsproperties | Gate |
|---|---|---|---|
| M1 Commit/Recovery | Sequencer, Stamp-Reserve, Redo-Gruppen, `durable_stamp`, Catalog, Crash/Open/Replay | nur vollständige durable Gruppen sichtbar; keine Stamp-Wiederverwendung; Recovery erzeugt gültigen Präfix | G-F/G-W |
| M2 Erasure/Restore | Ledger-Intent, Gate, Anchor, KeyDestroyIntent, Providerzustand, Purge-Batches, Backup-Restore | Intent vor Gate; kein Gate ohne wiederherstellbaren Tombstone; kein KeyDestroy vor Anchor; keine Sichtbarkeit vor Reconcile; `Done` nur nach Attesten | G-E |
| M3 Branch/MVCC | Parent-/Child-Branches, ForkPoint, Floors, Punkt-/Intervalländerungen, Guards | Fork-Sicht entspricht `fork_stamp`; keine Mutation unter `history_floor`; Range-Konflikte werden erkannt; Pins begrenzen GC | G-V |
| M4 Katalog/Purge | Catalog-Edits, Graveyard, Reader-Pins, Scratch, Unlink/Ack, Crash | Datei entweder lebend oder persistent im Graveyard; kein Scratch-Reuse vor UnlinkAck; kein Zwischenbatch als `Done` | G-E |
| M5 Extension/Index | Durable-Stamps, applied watermark, Gate-Generation, Kandidaten, Rewrite-Atteste | kein Ack über `durable_stamp`; Gatefilter vor Scoring/Traversal und vor Rückgabe; fehlender Pflichtbeleg verhindert Erfolg | G-E/G-P |
| M6 Restriction/Closure | Linkgraph, Generation, Restriction, Gate-Hülle, parallele Link-/Erase-Anfragen | keine partielle Kaskade; BLOCK_ERASE blockiert vor Mutation; Hülle ≤ Limit oder `EraseFanoutExceeded` | G-E |

## Pflichtabnahme

1. Jedes Modell hat `Spec`, `Init`, `Next`, Typinvarianten, Sicherheitsproperties und dokumentierte Fairnessannahmen.
2. TLC/Apalache-Konfiguration und exakter Aufruf werden versioniert. Zustandsraumgrenzen sind begründet; mindestens ein negatives Modell muss bei absichtlich verletzter Regel einen Counterexample liefern.
3. Für Commit/Recovery und Erasure/Restore werden Crash-Schnittpunkte zwischen jedem benachbarten dauerhaften Schritt modelliert.
4. Counterexamples werden als regressionsfähige Szenarien in FaultFs-/Integrationstests übertragen; ein Modellfehler wird nicht mit zusätzlicher Fairness kaschiert.
5. Release-Artefakte enthalten Toolversion, Konstanten, Property-Liste, Ergebnis und Modellhash. Fehlende oder nicht reproduzierbare Evidenz lässt das zugehörige Gate offen.

Modellcode wird in diesem Verzeichnis nach `M1-commit-recovery.tla`, …, `M6-restriction-closure.tla` abgelegt. Bis die Dateien samt reproduzierbarem Lauf vorhanden sind, ist der Modellstatus **offen**.
