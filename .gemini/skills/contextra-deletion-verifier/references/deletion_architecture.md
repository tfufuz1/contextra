# Referenz: Architektur der Löschpipeline

Nicht normativ. Maßgeblich: [`docs/SPEC.md`](../../../../../docs/SPEC.md), Kapitel 11, 12 und 13.

## Gate-Varianten (11.3)

- `Erased`: Lesen, Suchen, Schreiben und Zeitreisen sind gesperrt. Ein Eintrag im Gate-Log (`gates.log`) ist Voraussetzung.
- `Restricted` (Art. 18): Lesen und Schreiben gesperrt, nichts wird vernichtet. Die Kompaktion verwirft diese Daten nie (I-18). Aufhebung nur über `lift_restriction` mit Journal-Eintrag.

## Schlüsselebenen (D-79)

| Ziel | Slot, der vernichtet wird |
|---|---|
| Dokument | `D` |
| Betroffener (Subject) | `S` |
| Collection | `C` |
| Mandant | `T` |

Dateischlüssel (`F`) werden im Purge vernichtet, wenn die Datei ausscheidet (D-85).

## Key Coverage je Schicht (D-69, 12.5)

| Schicht | Nach `KeyDestroyed` | Danach |
|---|---|---|
| `doc_store` | `full` | – |
| `vector_payload` | `full` | – |
| `text_postings` | `unlinked` | `TextIndexUntilPurge` bis zum Segment-Rewrite |
| `vector_edges` | `none` | `IndexTopologyInference` bis zum Rebuild |
| `file_structure` | `none` | wertlos nach Dateischlüssel-Vernichtung |

## Invarianten (Auswahl aus 13.1)

- I-1: kein Schreiben in einen gegateten Bereich nach persistiertem Gate.
- I-2: ein gegateter Bereich ist auch nach Neustart `Erased`.
- I-8: `Requested` ist vor dem Gate dauerhaft.
- I-9: Tresor-Slot ist dauerhaft, bevor ein Record unter dem DEK dauerhaft wird.
- I-15: jede Löschung ist idempotent und nach Neustart fortsetzbar.
- I-16: keine Hintergrundoperation hält eine wartende Löschung länger als 1 s.
- I-20: die Sequenz ist über Restores monoton.

## Fehlermodi (Auswahl aus 13.2)

- FM-1 bis FM-3: Absturz zwischen Gate und Nullung, im Rewrite, Platte voll.
- FM-8: Restore erkannt, Löschungen werden wiederholt.
- FM-9: Journal ohne Gate-Log; das Gate wird aus dem Journal rekonstruiert.
- FM-14: Absturz nach Manifest-Swap, vor Nullung des `F`-Slots.

## Restrisiken (Anhang D)

`FlashRemanence`, `FsJournalOrCow`, `WrappedKeyRemanence`, `FreedBlockRemanence`, `OldKeyslotRemanence`, `ProcessMemory`, `IndexTopologyInference`, `TextIndexUntilPurge`, `ReplicaWindow`, `ExternalCopies`, `ExternalProcessors`, `PseudonymCommitment`, `KeyCompromise`.
