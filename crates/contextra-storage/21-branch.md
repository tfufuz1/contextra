# SP-BRANCH — Clusterlokale Branches, Diff und atomarer Merge

| | |
|---|---|
| Modul | `engine/branches.rs`, `SYS_BRANCH` |
| Besitzt | Branch-Metadatenformat, Fork/Drop/Diff/Merge-Plan, Branch-Caps |
| Konsumiert | PORTS, COMMIT, READ, CATALOG, CLUSTER, MEMVER |
| Spec-Bezug | R8 §8; INV-BR-1…4, INV-ERA-2 |
| Phase | P7 (G-V), Format-Slot ab G-F |

## 1 Auftrag
Branches sind clusterlokal; kein globaler DAG, Content Addressing oder Kern-Merge-Operator. Fork erzeugt nur Metadaten in O(1), erbt Elternkette und Stamp-Kappe. Branchdaten/-metadaten liegen im Cluster und werden mit ihm gelöscht.

## 2 Verträge
`fork(cluster,parent,ForkPoint,name,ttl)`, `diff(base,head)`, `merge(target,source,resolutions)`, `branch_drop`. Branch-ID wird vorab durable reserviert; SYS_BRANCH-Payload wird vor Rang 1 versiegelt. Commit-Stamp bleibt äußerer Frame-/Record-Stamp; bei `Head` ist er zugleich der `fork_stamp`, der versiegelte Payload braucht keinen unter Rang 1 unbekannten Stamp. Committer validiert Reservierung, Parent-Head und Cluster-Generation erneut. `ForkPoint::At(S)` verlangt `gc_floor ≤ S ≤ parent_head`; `Head` wird im Sequencer linearisiert. Merge-Plan bindet `source_head`, `target_head` und `merge_base`; Konflikt ⇒ keine Änderung. Erfolg schreibt Auflösungen und terminalen `Merged`-Status in genau einem Committer-Frame. Caller liefert Auflösungen.

## 3 Regeln
BranchId monotone, clusterlokal und nie wiederverwendet. Tiefe ≤ max_branch_depth; Fork pinnt `gc_floor ≤ fork_stamp` bis Branch terminiert. Erfolgreich gemergte Source-Branches sind terminal; für weitere Writes wird neu geforkt, daher `merge_base = fork_stamp` und keine mehrdeutige „letzte Merge“-Basis. Reads folgen Elternkette mit Stamp-Kappen und Versionpriorität. Merge-Konflikte schließen Punktmutationen und halboffene Intervalle aus `RangeDelete`/`EraseRange` ein; Guards werden beim Commit erneut validiert. Merge-Rebase/Strategie ist Sache Aufrufer/API, nicht Storage. Kein Cipher/KDF/KeyProvider unter Rang 1.

## 4 Invarianten / Umsetzung
INV-BR-1/2/3/4, INV-ERA-2, INV-LOCK-1/2. Referenzmodell vergleicht jede Branch-Key-Sicht; Fork O(1); Merge Konfliktatomarität und Intervallkonflikte; ForkPoint unter/über Floor; Race Gate vs. vorbereiteter versiegelter SYS_BRANCH-Frame.

## 5 Arbeitspakete
WP-21.1 SYS_BRANCH Format + ForkPoint/Merge-Basis (Slot G-F, Semantik P7) · WP-21.2 Reservation/Preseal/Fork/Drop/TTL (P1/P7) · WP-21.3 Branch read caps/Diff (P7) · WP-21.4 Punkt-/Intervallkonflikte + atomarer Merge-Frame (P3/P7). Abnahme: H-20…H-22 plus Modellgleichheit und Interleaving-Simulation.
