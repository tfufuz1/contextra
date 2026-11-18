# AGENTS.md — Contextra Workspace Richtlinie

**Status:** Normativ (Quelle der Wahrheit: `docs/SPEC.md` v1.0)  
**Datum:** 18. November 2026  
**Zielgruppe:** KI-Assistenten, autonome Agenten und Principal Engineers

---

## 1. Arbeitsanweisung & Geltung

Alle Agenten und Entwickler, die am Repository `contextra` arbeiten, sind strikt an die Festlegungen in `docs/SPEC.md` und den zugehörigen Crate-Spezifikationen in `docs/crates/` gebunden.

### Rangfolge der Dokumente:
1. `docs/SPEC.md` (Gesamtspezifikation — Single Source of Truth)
2. `docs/crates/*.md` (Spezifikationen pro Crate)
3. Diese `AGENTS.md` (Entwickler- & Agenten-Verhaltensrichtlinien)
4. Crate-lokale Dokumentationen / Readmes

---

## 2. Konsolidierte Architektur (23 Crates)

Der Workspace ist streng hierarchisch in vier Ringe/Ebenen gegliedert. Abhängigkeiten dürfen **ausschließlich abwärts** zeigen (E3 → E2 → E1 → E0).

```
┌─────────────────────────────────────────────────────────────┐
│ E3: Endpunkte & Werkzeuge                                   │
│     contextra-mcp, contextra-testkit, contextra-bench       │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│ E2: Integration & Orchestrierung                            │
│     contextra (Façade), contextra-db (Integrationsmotor),   │
│     contextra-engine (deprecated Façade),                   │
│     contextra-types, contextra-ports                        │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│ E1: Autonome Domänenbausteine                               │
│     contextra-store (LSM), contextra-vector (HNSW),         │
│     contextra-text (BM25), contextra-graph (GraphRAG),       │
│     contextra-rank (Fusion), contextra-checkpoint,          │
│     contextra-crypto (AEAD/HKDF), contextra-mvcc            │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│ E0: Autarke Fundamente                                      │
│     contextra-ids, contextra-env, contextra-sys (unsafe),   │
│     contextra-durable-fs, contextra-simd (unsafe),          │
│     contextra-privacy, contextra-erasure                    │
└─────────────────────────────────────────────────────────────┘
```

### Entfernte Crates (Scope-Reset)
Die folgenden 13 Crates wurden zur Stabilisierung und Verhinderung von Architekturlecks aus dem Workspace entfernt:
- `contextra-cognition`, `contextra-agent`, `contextra-adapt`
- `contextra-wire`, `contextra-py`, `contextra-prompt-guard`
- `contextra-ingest`, `contextra-infer-onnx`, `contextra-infer-ollama`
- `contextra-infer-candle`, `contextra-sandbox`, `contextra-router`
- `contextra-kv-cache`

> [!CAUTION]
> **Reaktivierungsverbot:** Kein entferntes Crate darf ohne Bestehen des formalen Gates `G-REACTIVATE` (siehe `DECISIONS_SCOPE_REDUCTION.md` bzw. `docs/SPEC.md` §2.2) reaktiviert oder als Workspace-Member eingetragen werden. `contextra-vault` verbleibt dormant in `exclude`.

---

## 3. Datenklassifikation & DSGVO-Löschkette

Contextra implementiert belegbare DSGVO-Löschung mittels **Krypto-Shredding** kombiniert mit **physischer Schichtenbereinigung**.

### 3.1 Datenklassen
- **Klasse K (Unternehmenswissen):** Verträge, Handbücher, Fachdaten. Geschützt mit Collection-KEK. K-Schreibpfade akzeptieren nur `Validated<T>` (`contextra-privacy`), um unerkannte PII auszuschließen.
- **Klasse P (Personenbezogene Daten):** Daten natürlicher Personen. Streng gekapselt! Adressierung erfolgt ausschließlich über `SubjectRef` (128-Bit-Zufalls-ID). Verschlüsselung mit individuellem Betroffenen-Schlüssel (`SubjectKey` → `SubjectDocDEK`).
- **Klasse D (Abgeleitete Daten):** Vektoren, Volltext-Postings, Graph-Entitäten, Caches. Tragen verpflichtend `Lineage` zum Quelldatum zur kaskadierten Bereinigung.
- **Klasse E (Löschmaterial):** Audit-Trail, Journal, Ledger, signierte Berichte.

### 3.2 Die 6-Schritte-Löschsaga (`contextra-erasure`)
1. **Intent & Validierung:** `RetentionAttestation` prüfen, Journal-Append (fsync #1).
2. **Gate setzen:** `CommitBarrier::exclusive()`, `ReadGate` aktivieren (fsync #2).
3. **Schlüsselvernichtung:** Betroffenen-/Dokumentenschlüssel im Tresor überschreiben, nullen, fsync #3. `verify_key_absent` bestätigt Abwesenheit. Ab hier kryptographisch unlesbar!
4. **Physische Bereinigung (Purge):** Exklusiv unter `PurgeBarrier` in Reihenfolge:
   - Snapshot-Widerruf (`mvcc`)
   - Text-Segment-Rewrite (`text`)
   - WAL-Drain & SSTable-Rewrite (`store`)
   - HNSW-Graph-Reparatur (`vector`)
   - Graph-Bereinigung (`graph`)
   - Slot-Bereinigung (`crypto`)
   - Cache-Invalidierung (alle Layer)
5. **Messung & Positivkontrolle:** Scanner-Kalibrierung mit Canary-Mustern. Alle Layer-Witnesses müssen `residue_matches == 0` melden.
6. **Bericht & Ledger:** Ed25519-signierter Bericht und kryptographisch verketteter Ledger-Eintrag außerhalb des Datenverzeichnisses.

### 3.3 ReadGate-Invariante
Jeder Lesepfad (`get`, `scan`, `search`, `hybrid_search`, `graph_query`, `export`, `mcp`) MUSS synchron gegen das `ReadGate` prüfen. Bei `Erased` erfolgt sofortiger Abbruch ohne Daten- oder Metadatenlecks (Fail-Closed).

---

## 4. Entwicklungsstandards & Invarianten

1. **Unsafe-Inseln:** `#![forbid(unsafe_code)]` ist im gesamten Workspace aktiv. Einzige Ausnahmen: `contextra-sys` (OS-Primitive) und `contextra-simd` (SIMD-Intrinsics).
2. **Zero-Panic-Policy:** Keine Verwendung von `unwrap()`, `expect()`, `panic!()`, `todo!()` oder `unimplemented!()` in Nicht-Test-Code. Alle Fehlerpfade verwenden `Result` mit `ContextraError` bzw. crate-spezifischen Fehlertypen.
3. **Keine stillen Fallbacks:** Fehlt ein Schlüssel oder scheitert eine kryptographische Operation, MUSS ein Fehler zurückgegeben werden. Es gibt keine Fallback-Schlüssel, keine Null-Schlüssel und keine deterministischen Notfall-Passphrasen.
4. **Zeroize:** Sämtliches Schlüsselmaterial MUSS `ZeroizeOnDrop` implementieren.
5. **Determinismus:** Gleiche Eingaben müssen zu identischen Such- und Ranking-Ergebnissen führen (Tie-Breaking nach `DocId` aufsteigend).
6. **Feature-Hygiene:** `default`-Features müssen in allen Crates leer sein.
