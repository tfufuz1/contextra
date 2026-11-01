# AGENTS.md — `contextra-mvcc`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.7](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-mvcc` stellt deterministische Multi-Version Concurrency Control (monotone Sequenzverwaltung, Snapshot-Registry, Write-Skew-Erkennung via Serializable Snapshot Isolation / SSI und Snapshot-Widerruf bei Löschungen) bereit.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 2 (Transaktionsisolation)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`
- **Verbotene Abhängigkeiten:** Keine Abhängigkeiten zu Ring 3 oder 4, keine direkte I/O-Abhängigkeit.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Kein Halten von Sperren über `.await`-Punkte hinweg.
  - Keine Verwendung von `u64::MAX` als Sentinel-Sequenznummer (B-11).

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.7)

| Modul | Inhalt & Verantwortung |
|---|---|
| `seq` | Monotone Sequenznummern: `Seq(NonZeroU64)`. Trennung von Sequenzen und Tombstone-Flags; kein Sentinel-Wert. |
| `snapshot_registry` | Registrierung aktiver Snapshots: `register`, `release`, `min_active() -> Option<Seq>`, `revoke_covering(&Matcher)`. |
| `ssi` | `SsiValidator`: Konflikterkennung für Write-Skew; `prune_through(bound: Seq) -> Result<usize, MvccError>`. |
| `tx_buffer` | Transaktionspuffer zur Isolation uncommitteter Änderungen. |

## 4. Kritische Invarianten & Bugfixes
1. **Behebung Befund D-C02 (Snapshot-Sentinel):**
   - In `ssi.rs:640–652` darf `prune_through` ein gesetztes Tombstone-Bit im Eingabewert **niemals** stillschweigend maskieren (`& !TOMBSTONE_BIT`). Ein solches Bit ist ein Programmierfehler und führt zwingend zu einem `MvccError::InvalidSequence`.
   - `pruned_through` steigt strikt monoton und überschreitet niemals die höchste committete Sequenznummer.
2. **Snapshot-Widerruf bei Purge (`revoke_covering`, D-11):**
   - Alle aktiven Snapshots, die gegatete oder gelöschte Schlüssel sehen könnten (`seq <= erased_through_seq`), werden als widerrufen markiert.
   - Jede Folgeoperation auf einem widerrufenen Snapshot scheitert fail-closed mit `ContextraError::SnapshotRevoked`.
3. **Kein Deadlock:** Registry-Operationen sind lock-frei oder verwenden kurzlebige `parking_lot`-Mutexes ohne I/O.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-2.1: Snapshot-Sentinel reparieren (D-C02)**
  - Bereinigung von `ssi.rs` (Z. 640–652): Fehler statt stiller Maskierung, Monotonie absichern.
- **[ ] WP-2.2: Sequenzsemantik festschreiben**
  - Durchgängige Einführung von `Seq(NonZeroU64)`; Snapshots sind `Option<Seq>`.
- **[ ] WP-3.9: Snapshot-Widerruf implementieren (D-11)**
  - Implementierung von `SnapshotRegistry::revoke_covering(&Matcher)`.

## 6. Verifikation & Tests
- **T-SEQ-1 / T-SEQ-2:** Sequenzmonotonie und Pruning-Verhalten.
- **Regressionstest D-C02:** "Flush ohne Snapshot, danach gelingt Commit" (darf nicht mehr mit 'snapshot too old' scheitern).
- **Loom-Verifikation:** Nebenläufigkeitstests für die `SnapshotRegistry`.
- **Property-Testing:** Validierung gegen `ReferenceModel` aus `contextra-testkit`.
