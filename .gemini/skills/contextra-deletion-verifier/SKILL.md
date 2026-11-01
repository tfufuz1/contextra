---
name: contextra-deletion-verifier
description: >-
  Use this skill to verify, test, and develop the Contextra deletion pipeline, ErasureSaga state machine,
  Ebene A physical purge, ReadGate (ErasedSet), E1/E2/E3 residue scans, Canary tests, and Ed25519 erasure reports.
---

# Contextra Deletion Verifier – Löschpipeline & DSGVO-Konformität

Dieser Skill führt Agenten und Entwickler durch die Entwicklung, Verifikation und Auditierung der gemessenen Löschpipeline nach [docs/SPEC.md §11, §12](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [ADR-001](file:///home/freddy/Projekte/contextra/docs/SPEC.md#L1421-L1434).

---

## 1. Architektur-Prinzip (ADR-001, D-01)

Gemäß **ADR-001 (SPEC §3.3, D-01)** entfällt die sogenannte Ebene B (kryptografische `DocDek`-Schlüsselvernichtung) in v1 vollständig.

Löschung in Contextra v1 ist die **physische Bereinigung (Ebene A)** kombiniert mit:
1. **Fail-Closed `ReadGate` (`ErasedSet`)**: Sperrt Lese- und Suchpfade sofort bei Status `Gated`.
2. **Snapshot-Widerruf (`revoke_covering`)**: Widerruft aktive MVCC-Snapshots, die Zielversionen sehen könnten.
3. **Physischer Purge**:
   - Memtable: vollständiges Entfernen aller Versionen (`remove_range`).
   - Store: WAL `drain`, SSTable-Rewrite (`rewrite_file`) ohne Zieldaten unter Beibehaltung der Originalsequenzen, atomarer `swap` via Manifest, Block-Cache Eviction mit Zeroize.
   - Vector: HNSW Slot-Zeroing (`zero_slot`), Snapshot-Chunk-Rewrite, Batch-Rebuild.
   - Text: Puffer leeren, Segment-Rewrite, Alt-Segment löschen.
4. **Maschinelle Residuenmessung (E1, E2, E3)** mit obligatorischer Positivkontrolle (Canary).
5. **Kryptografischer Löschbericht (`ctx-erasure-report/1`)**: Ed25519-signiert, maschinenlesbar, ohne personenbezogene Daten.

---

## 2. Die Lösch-Saga Zustandsmaschine (SPEC §8.2, §11.2)

Jede Löschung läuft deterministisch und crash-tolerant als Saga in `contextra-erasure` ab:

```
[Requested]
     │
     ▼
  [Gated] ────────► Sofortige Lesesperre via ErasedSet (fail-closed)
     │              Widerruf betroffener MVCC-Snapshots (SnapshotRevoked)
     ▼
 [Drained] ───────► WAL versiegelt, Memtables geflusht, temporäre Puffer geleert
     │
     ▼
[Rewritten] ──────► SSTables, Textsegmente und Vektorsnapshots ohne Zieldaten neu geschrieben
     │              Atomarer Manifest-Swap, Blockcache gelöscht & genullt
     ▼
 [Cleaned] ───────► Alt-Dateien gelöscht und Fehlen via Vfs::exists verifiziert
     │
     ▼
 [Measured] ──────► Residuen-Scan (E1 Rohscan, E2 Decoder-Scan, E3 Schlüsselabwesenheit)
     │              mit vorangestellter Canary-Positivkontrolle
     ▼
[Completed] ──────► Ausstellung des signierten Berichts ctx-erasure-report/1 (Ed25519)
```

### Crash-Recovery Invariante (I-11)
- Bei einem unvorhergesehenen Absturz setzt `ErasureSaga::recover()` vor der Freigabe von Schreib- oder Lesepfaden auf.
- Jeder Fehler in einer Phase persistiert den Zustand `Pending { attempt, next_retry_at }` mit exponentiellem Backoff (1 s bis max. 15 min). Fehler werden niemals ignoriert.

---

## 3. Messklassen & Nachweis (SPEC §12)

- **E1 (Rohscan)**: Byte-genauer Scan aller Dateien im Datenverzeichnis nach Ziel-Needles (BLAKE3-Hashes, Schlüsselbytes).
- **E2 (Decoder-Scan)**: Strukturierter Scan nach vollständiger Entschlüsselung aller SSTables, Textsegmente und Vektorsnapshots.
- **E3 (Schlüssel-/Slot-Abwesenheit)**: Verifikation, dass Schlüssel, Locator-Einträge und Slots nicht mehr existieren.
- **Positivkontrolle (Canary)**: Vor jeder Messung injiziert `CanaryFactory` Test-Needles in einen Testmandanten, um die Detektionsfähigkeit der Scanner maschinell zu beweisen.
