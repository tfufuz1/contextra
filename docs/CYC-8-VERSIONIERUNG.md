# CYC-8 — Versionierung (Branches, Chain-Resolve, Diff, Merge, Historie, Zeitmarken)

Ersetzt R14 §7 und §11.14 (Branch-, Zeit- und Historienteil; Retention-Semantik bleibt in CYC-3 §10). Abhängigkeiten: SYS-2 §3/§5.2/§5.8/§5.4 (Schlüsselordnung, `control.op`, `SYS_BRANCH_*`, `ChangeIndexBlock`, `BranchTableBlock`), SYS-3 §7 (`Branches`), SYS-4 §2 (Limits), CYC-1 (Sequencer, Guards, `IdemToken`), CYC-2 §4 (Chain-Resolve), CYC-3 (Branch-Löschung), CYC-5 (Versions-GC, Flush, Idem-Lane), CYC-6 (Open/Replay). Gate: **G-V**.

## 1. Zweck und Grenzen

Der Zyklus legt fest, wie ein Cluster in O(1) verzweigt, wie ein Branch gelesen, verglichen, zusammengeführt und beendet wird und wie lange Versionen für Zeitreise erreichbar bleiben. Er legt außerdem das Zeitmodell des Kerns fest (Stamp = Transaktionszeit) und die einzige Brücke zwischen Wanduhr und Stamp (`time_mark`).

**Kernsätze.**
1. Ein Branch ist clusterlokal und stirbt mit dem Cluster. Es gibt keinen instanzweiten Commit-DAG und keine Inhaltsadressierung: sie würden Löschdomänen über Hashes verbinden (`INV-PRIV-1`, Cluster = Löschdomäne).
2. Fork ist eine Metadaten-Operation (ein Control-Frame, keine Datenkopie). Seine Kosten hängen weder von der Datenmenge noch von der Branch-Anzahl ab.
3. Kein Branch kann den clusterweiten Löschzustand aufheben: Gate, `EraseRange`-Masken und Restriction wirken **vor** jeder Versionsauflösung und über alle Branches (`INV-ERA-2`, CYC-2 §2).
4. Merge ändert nie still: Konflikte werden als Wert geliefert und schreiben nichts; ein Merge ist ein normaler `Change` mit Guards und `IdemToken`.
5. Zeitreise ist Lesen oberhalb `history_floor`. Unterhalb gibt es `BelowFloor`, nie ersatzweise Daten und nie stilles Rebasen (`INV-HIST-1`).
6. Keine Wanduhr im Hot Path. Wanduhr↔Stamp existiert nur als grobe, monotone Markenreihe (§7.3).

Nicht Teil: Typisierte Merge-Strategien (LWW, Union, Textmerge, semantisch) liefert der Client; Retention-Fristen (CYC-3 §10); Backup (CYC-6).

## 2. Akteure und Zustandsautomat

| Akteur | Aufgabe | Rang/Thread |
|---|---|---|
| Aufrufer (`Branches`-Fassade) | Reservierung, Validierung ohne Sperre, Einreichen | Aufrufer |
| Sequencer | Linearisierung von Fork/Merge/Drop, Revalidierung | Rang 1 (CYC-1 §3) |
| Branch-Tabelle (RAM, je Cluster) | `BranchEntry{state, parent, fork_stamp, ttl_deadline, children, created_by}` | Rang 6 je Shard (SYS-1 §5) |
| Lane `idem` (CYC-5) | TTL-Ablauf, `time_mark` | Wartungsthread |
| Flush/Compaction | persistiert `BranchTableBlock`, `ChangeIndexBlock`; GC unter `floor` | Wartung |

**Zustände eines Branches** (`SYS_BRANCH_*`-Kopf, SYS-2 §5.8):

```
(Reserviert) ──Fork-Frame──► Live ──Merge-Frame (als Source)──► Merged ──branch_drop──► Dropped
                               │                                                         ▲
                               └───────────────────branch_drop (leaf)──────────────────┘
Branch 0 (main): immer Live; kein Fork-Frame; weder droppbar noch mergebar als Source.
```

| Zustand | Lesen | Schreiben | Fork als Parent | Merge als Source | Merge als Target |
|---|---|---|---|---|---|
| `Live` | ja | ja | ja | ja | ja |
| `Merged` | ja (read-only bis Drop) | `BranchClosed` | `BranchClosed` | `BranchClosed` | `BranchClosed` |
| `Dropped` | nein (`UnknownBranch`) | `BranchClosed` | `BranchClosed` | `BranchClosed` | `BranchClosed` |
