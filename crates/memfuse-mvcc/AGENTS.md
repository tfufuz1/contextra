# MemFuse — AI-Assistenten-Kontext (`memfuse-mvcc`)

## Verifizierter Codestand · Ring 0 (Fachkern)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse-mvcc`.
> `memfuse-mvcc` stellt MVCC-Primitiven bereit (SeqLog, SnapshotRegistry, TxBuffer).
> Er ist strikt synchron (P26) und erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 0 Fachkern**:
  - `seq_log`: Sequentielles Änderungsprotokoll.
  - `snapshot`: Snapshot-Isolation und Read-Pinning.
  - `tx_buffer`: Transaktionales Staging und Orphan-Reaper.
