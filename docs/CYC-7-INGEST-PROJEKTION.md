# CYC-7 — Ingest und Projektion (Publish-Record, Feed, Partitionen, Generationen, Modi A/B/C, Pseudonymisierung, Analyzer, Graph, Person → Cluster)

Ersetzt R14 §11.1–11.9 und §11.12–11.13 (Query-Plan/Fusion/Budget: CYC-2; Branch/Zeit/Retention: CYC-8, CYC-3 §11). Abhängigkeiten: SYS-1 §1/§3 (Topologie, Ports), SYS-2 §5.8/§5.9 (Schlüsselschema, `head`, Directory, Segmentcontainer), SYS-3 §7 (`Feeds`), SYS-4 §2.6 (Retrieval-Zahlen), SYS-5 §3/§5/§7 (Mandant, Egress, Betroffenenrechte), CYC-1 (Commit, `IdemToken`), CYC-2 §3/§5 (Sichtbarkeit, Query), CYC-3 (Gate, Ledger, `ExtAck`), CYC-4 (Partition-/Entry-Schlüssel), CYC-5 (Rebuild-Lane, Budget), CYC-6 (Restore). Gate: **G-R1** (Feed/Projektion), **G-R2** (Partitionen/Generationen, Vektor, Text, Graph), **G-R3** (Query-Plan, in CYC-2), **G-R4** (Pseudonymisierung, Verzeichnis, Egress).

## 1. Zweck und Grenzen

Der Zyklus beschreibt, wie aus kanonischen Records des Kerns abgeleitete Strukturen (Vektorindizes, Postings, Graphsegmente) entstehen, löschbar bleiben und nie über die Wahrheit hinausreichen; wie personenbezogene Daten getrennt werden; und wie die Zuordnung Person → Cluster löschbar bleibt. Er ist die Brücke zwischen Kern (`storage`) und Retrieval-Crates (`vector`, `text`, `graph`, `rank`); die Bindung geschieht **ausschließlich** in `engine::projection` (SYS-1 §1).

**Kernsätze.**
1. **Wahrheit = Records im Kern; Ableitung = alles andere** und jederzeit aus der Wahrheit neu aufbaubar (EX-1). Kein Index entscheidet über Commit (EX-2).
2. Jede Ableitung ist entweder `ClusterSections` im Kern oder mandantenreiner `Rebuildable`-Owner mit Generationsschlüssel (EX-4). Geteilte Strukturen enthalten nie Personen-Klartext-Ableitungen (`INV-SEP-2`).
3. Jeder Kandidat passiert den Kern-Gate-Zustand und `filter_visible` vor Scoring **und** vor Ausgabe; Owner-Masken beschleunigen, begrenzen nie die Korrektheit (EX-5, `INV-VIS-1`).
4. Ein Index-Watermark reicht nie über `durable_stamp` derselben Instanz (`INV-RAG-2`); Lücken sind Delta-Pfad oder `IndexLag`, nie stilles Auslassen (`INV-RAG-3`).
5. Löschung einer Person ist nur vollständig, wenn Kern **und** alle Owner (Index-Partitionen, Graph, Directory) ihre `EraseAttestation` liefern; fehlende Atteste verhindern systemweites `Complete` (`INV-ERA-8`).

Nicht Teil: Fusion, Tokenbudget, Query-Plan (CYC-2); Löschprotokoll (CYC-3); Produktpolitik der Chunk-Größen, Modellwahl, Prompt-Konstruktion (obere Schicht).

## 2. Grundregeln, Akteure, Schlüsselschema

### 2.1 Regeln EX-1…EX-9

| ID | Regel |
|---|---|
| EX-1 | **Wahrheit** = kanonische Records im Kern (atomar, durabel, cluster-sealed). **Ableitung** = Indizes, Graphen, Präfixsummen, Caches. |
| EX-2 | Kein Schreibpfad ändert zuerst einen Index und dann den Kern. Kein 2PC, kein `CommitLedger`, keine Kompensation. |
| EX-3 | `vector`, `text`, `graph`, `rank` sind persistenzneutral und kennen weder `storage` noch einander. Bindung nur in `engine::projection`. |
| EX-4 | Abgeleitet = `ClusterSections` oder mandantenreiner `Rebuildable`-Owner mit Generationsschlüssel; anderes ist für aus Nutzdaten abgeleitete Strukturen unzulässig. |
| EX-5 | Jeder Kandidat passiert Kern-Gate und `View::filter_visible` vor Scoring und vor Ausgabe. `on_gate`-Masken sind Beschleuniger (FIX-11). |
| EX-6 | Watermark = `(instance_id, applied_stamp)`, nie über `durable_stamp`; keine instanzübergreifenden Vergleiche (`INV-RAG-2`). |
| EX-7 | Quelldaten und Indexgeneration sind nicht gemeinsam atomar. Vertrag: Durability, Wiederaufbau, Idempotenz, Sichtbarkeit, Löschbeleg; kein Null-Lag. |
