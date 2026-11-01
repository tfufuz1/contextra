# Contextra Deletion Pipeline Architecture Reference

> **Normative Vorgabe gemäß docs/SPEC.md §11, §12 und ADR-001 (D-01).**

## 1. Löschmodell & ADR-001 (D-01)

Gemäß **ADR-001 (SPEC §3.3, D-01)** entfällt die sogenannte Ebene B (kryptografische DocDek-Schlüsselvernichtung) in v1.
Die Löschung basiert rein auf:
1. **Physischer Bereinigung (Ebene A)**: Memtable Purge, SSTable-Rewrite/Swap, HNSW-Slot-Zeroing, Text-Segment-Rewrite.
2. **Fail-Closed ReadGate (`ErasedSet`)**: Sperrt Lese- und Suchzugriffe ab Status `Gated`.
3. **Snapshot-Widerruf (`revoke_covering`)**: Widerruft aktive MVCC-Snapshots.
4. **Maschineller Residuenmessung (E1, E2, E3)** mit Canary-Positivkontrolle.
5. **Ed25519-signiertem Löschbericht (`ctx-erasure-report/1`, D-13)**.

---

## 2. Messklassen nach SPEC §12.1 (D-35)

- **E1 (Rohscan)**:
  - Byte-genauer Scan aller Dateien im Datenverzeichnis nach Ziel-Needles (BLAKE3-Hashes, Tags, Schlüsselbytes).
- **E2 (Decoder-Scan)**:
  - Strukturierter Scan nach vollständiger AEAD-Entschlüsselung aller Container (SSTables, Textsegmente, Vektorsnapshots).
- **E3 (Schlüssel- und Slot-Abwesenheit)**:
  - Nachweis, dass der Zielschlüssel im Index, Locator und Tresor nicht mehr existiert.

---

## 3. Invarianten des Löschpfads

- **INV-FAILCLOSED-1**: Bei unbekanntem Zustand oder Fehlern verweigert das System den Zugriff fail-closed.
- **INV-NO-RESURRECTION-1**: Ein gelöschter Scope darf nach einem Crash, Restore oder Replay niemals wieder lesbar werden (`erasure/ledger.log` Restore-Guard).
- **INV-CLAIMS-1 (D-35)**: Kein Bericht ohne maschinelle Zeugen (`LayerScan`). Jede Behauptung beruht auf echten Messungen.
- **VETO-F02**: HNSW-Löschung erfolgt durch Tombstone-Markierung, Slot-Nullung (`zero_slot`) und periodischen Batch-Rebuild. Kein partielles Rewiring.

---

## 4. Schlüsselhierarchie (SPEC §10.1, §13.1)

```
Passphrase (Argon2id: m >= 64 MiB, t >= 3, p >= 1, Salt[32])
   │
   ▼
K_master (SecretKey32)
   │
   ├── HKDF("ctx/enc/v1")     ──► K_enc (SSTable- & Segment-AEAD)
   ├── HKDF("ctx/idx/v1")     ──► K_idx (blind_tag BLAKE3-Key)
   ├── HKDF("ctx/wal/v1")     ──► K_wal (WAL-HMAC-Kette)
   ├── HKDF("ctx/journal/v1") ──► K_journal (Löschjournal-AEAD)
   ├── HKDF("ctx/vault/v1")   ──► K_vault (Tresor-AEAD)
   ├── HKDF("ctx/unit/v1")    ──► K_unit (Envelope-Sealing)
   └── HKDF("ctx/sign/v1")    ──► K_sign_seed (Ed25519-Berichtssignatur)
```
