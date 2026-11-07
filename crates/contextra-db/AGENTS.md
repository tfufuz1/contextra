# AGENTS.md — `contextra-db`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-db` ist die High-Level Datenbank-Orchestrierungsschicht (Ring 3) für Multi-Signal-Fusion (Reciprocal Rank Fusion über Vektor, Volltext, Graph und Storage), Markdown-Chunking, Token-Budget-Management und iterative Suche.
Als modularer Baustein orchestriert er die Kernsignale über Ports und traits, ohne proprietäre oder interne Engine-Typen nach außen lecken zu lassen.
Im Baukastensystem wurde die monolithische Löschkaskade vollständig entfernt: Lösch- und Nachweispfade delegieren an den autarken Baustein `contextra-erasure`.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 3 (Main Database Abstraction & Hybrid Search)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-engine`, `contextra-sys`, `contextra-types`, `contextra-ports`, `contextra-vector`, `contextra-store`, `contextra-graph`, `contextra-text`, `contextra-adapt`, `contextra-rank`, `contextra-checkpoint`, `contextra-cognition` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Kausalität & Transaktionen:** `TxId` wird deterministisch und monoton bezogen; Kausalitätsbrüche bei Graph- und Speicherindizes sind ausgeschlossen.
- **4-Signal Fusion Pipeline:** Standardisierte Normalisierung via RRF (`reciprocal_rank_fusion`) zum Ausgleich von Score-Verzerrungen aus unterschiedlichen Domänen.
- **Strukturiertes Chunking:** Sauberes Aufteilen von Markdown-Dokumenten entlang semantischer Headings (`MarkdownChunker`).
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; robuste Handhabung leerer Trefferlisten in Fusionsalgorithmen.

## 4. Verbotene Muster
- Halten von Locks über `.await`-Punkte hinweg (Fehlerklasse 11 strikt verboten).
- Monolithische Löschketten im Crate (Delegation an `contextra-erasure`).
- Aufruf veralteter API-Methoden (siehe DEBT-05).
- Verwenden von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).

## 5. Lock-Hierarchie & Nebenläufigkeit
- **Strikte Lock-Reihenfolge zur Deadlock-Prävention:**
  1. `collections` (RwLock — Registry der aktiven Collections)
  2. `kv_locks` (KvKeyLocks — Sharded Key Locks pro Collection)
  3. `embedder` (RwLock — Lazy Embedder Initialization)
- **Regel:** Niemals Locks in abweichender Reihenfolge akquirieren. Langlaufende Inferenz-Tasks niemals unter Locks ausführen.

## 6. Tests, Sabotage & Qualitätssicherung
- **RRF Bounds & Edge Cases:** Sabotage-Tests mit komplett leeren oder disjunkten Signal-Listen zur Prüfung auf Out-of-Bounds Fehler.
- **Concurrency & Deadlock-Prüfung:** Parallele Lese- und Schreib-Transaktionen unter extremer Nebenläufigkeit.
- **Fuzzing (REQ-QA-003):** Markdown-Chunker Fuzzing gegen maliziös geschachtelte Header.

## 7. Offene Punkte & Technische Schulden
- **DEBT-05:** Rückbau der 40 dokumentierten Legacy-API-Aufrufstellen gemäß Migrationsplan.
