---
name: contextra-deletion-verifier
description: >-
  Use this skill to verify, test, and develop the Contextra 6-layer deletion pipeline, ErasureSaga state machine,
  Ebene A (in-memory) and Ebene B (cryptographic DocDek key revocation) erasure, Ed25519 proofs, and mutation tests.
---

# Contextra Deletion Verifier – Löschpipeline & DSGVO-Konformität

Dieser Skill führt Agenten und Entwickler durch die Entwicklung, Verifikation und Auditierung des zweistufigen Löschvertrags (Ebene A und Ebene B) nach [docs/SPEC.md §12](file:///home/freddy/Projekte/contextra/docs/SPEC.md#L405) und [PLAN.md §10](file:///home/freddy/Projekte/contextra/PLAN.md#L405).

---

## 1. Architektur-Prinzip: Zwei Ebenen

Schlüsselvernichtung allein gilt niemals als Löschung! Löschung wirkt ausschließlich durch das Zusammenwirken von:

| Ebene | Zielbereich | Methode | Messung / Nachweis |
|---|---|---|---|
| **Ebene A** | RAM, HNSW-Slots, BM25-Postings, Caches | Physisches Zeroizing, Segment-Rewrite, Cache-Drop | Positivkontrolle (Canary-Scan), Slot-Scan |
| **Ebene B** | Persistente SSTables, WAL, Snapshots, CoW/Flash | Vernichtung des `DocDek` (Dokument-Schlüssel) | KeyTable-Abwesenheit, scheiternde Entschlüsselung |

---

## 2. Die Lösch-Saga Zustandsmaschine

Jede Löschung muss deterministisch und crash-tolerant als Saga ablaufen:

```
[Requested]
     │
     ▼
  [Frozen] ────► Sofortige Lesesperre via ErasedSet (fail-closed)
     │
     ▼
[LogicallyDeleted]
     │
     ▼
[PlaintextPurged] ──► Ebene A: HNSW zeroize_and_free_node, BM25 Rewrite, Memtable
     │
     ▼
[PlaintextMeasured] ─► Bestätigung der Klartextfreiheit über LayerScanner
     │
     ▼
 [KeyDestroyed] ────► Ebene B: DocDek im KeyTable atomar überschreiben/löschen
     │
     ▼
  [Measured] ────────► Schlüsselabwesenheit und Canary-Entschlüsselung fehlgeschlagen
     │
     ▼
 [Completed] ───────► Ausstellung des signierten Proof v4/v5 (Ed25519)
```

### Crash-Recovery Invariante (INV-NO-RESURRECTION-1)
- Ein Absturz an jedem beliebigen Punkt führt bei `ErasureSaga::recover()` zum Wiederaufsetzen oder zum Verharren in `Partial{outstanding}`.
- Es darf niemals vorkommen, dass ein abgebrochener Commit nach einem Neustart wieder lesbar wird (D-01 Fix).

---

## 3. Verifikations-Schritte für den Entwickler

### Schritt 1: Scope-Prüfung (Kein Prefix-Matching auf Mandanten-Strings)
Prüfe in `crates/contextra-store/src/scanner/purge.rs`:
- Ziel muss ein strikter `ErasureScope { tenant, collection, doc }` sein.
- Der Bug F01 (LsmPurger ignoriert `_target` und purged alle Collections des Mandanten) muss durch gezielte Scans verhindert werden.

### Schritt 2: SSTable-Rewrite Atomarität (F02)
Prüfe beim Umschreiben von SSTable-Segmenten:
- Der alte SSTable-Reader bleibt aktiv, bis die temporäre Datei vollständig geschrieben, per `sync_all` gesichert und atomar umbenannt (`rename`) wurde.
- Fehlerhafte Schreibvorgänge dürfen keine Daten still verlieren.

### Schritt 3: Positivkontrolle (Canary-Test)
1. **Vor Vernichtung**: Test prüft, dass der Canary-Datensatz im Index findbar und entschlüsselbar ist.
2. **Nach Vernichtung**:
   - Ebene-A-Scan findet keinen Klartext-Match.
   - Ebene-B-Probe scheitert mit `AuthenticationTagMismatch` oder `KeyNotFound`.
3. Der Canary landet **niemals** im produktiven WAL.

### Schritt 4: Ed25519 Löschbericht (Proof v4/v5)
Prüfe den generierten Löschbericht in `crates/contextra-crypto/src/deletion_proof/`:
- **Nur asymmetrische Ed25519-Signatur** (`verify` benötigt nur den Public Key).
- Keine Klartext-IDs, personenbezogene Daten oder Prompt-Inhalte im Bericht (L-05).
- Restrisiken müssen explizit im Bericht deklariert sein (`UNMEASURED_ERASURE_LAYERS`: Blob, Checkpoints, Swap).

---

## 4. Testausführung

```bash
# Deletion E2E Tests
cargo test -p contextra-store --test deletion_e2e
cargo test -p contextra-crypto --test deletion_proof_tests
cargo test -p contextra-engine --test review_stub_proof

# Regressionstests für F01 und F02
cargo test -p contextra-store test_r_f01_scope_isolation
cargo test -p contextra-store test_r_f02_sstable_rewrite_atomic
```

Weitere Details: siehe [references/deletion_architecture.md](file:///home/freddy/Projekte/contextra/.agents/skills/contextra-deletion-verifier/references/deletion_architecture.md).
