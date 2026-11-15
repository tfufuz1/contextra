# Extraktion fuer `acta-history`

**Aufgabe:** Versionskontrolle K: Objekte, Refs, Sichtbarkeitsmengen, Diff, Merge, Takedown  
**Level:** L3 · **Budget:** 6500 LOC · **Spezifikation:** docs/SPEC.md Kap. 6  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-pii, acta-ledger, acta-vault, acta-seg, acta-gate  
**Umfang des Archivs:** 16 Dateien, 6499 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `mvcc` | Nur Referenz: tx_seq (6.3) und Schnappschuss-Registry (3.6). Git-Objektmodell (6.1) ist Neubau |
| `checkpoint` | Nur Referenz: Schnappschuss-/Zeitreise-Muster |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 5 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 7 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 90 | tokio/async (G-NOTOKIO: Kern synchron) |
| panic | 137 | unwrap/expect/panic (G-LINT) |
| hub | 10 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 2 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |
| liste_a | 2 | verbotene Formulierung (1.6, Liste A) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `blake3`, `bytes`, `parking_lot`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-history/`) | LOC | Befunde |
|---|---:|---|
| `checkpoint/src/guard.rs` | 856 | float=4, async=26, panic=13 |
| `checkpoint/src/hardlink_cloner.rs` | 295 | serde=2, async=10 |
| `checkpoint/src/manifest.rs` | 260 | serde=1, panic=2 |
| `checkpoint/src/meta.rs` | 271 | serde=3, panic=5, liste_a=2 |
| `checkpoint/src/store.rs` | 961 | float=1, serde=1, async=30 |
| `checkpoint/tests/guard_proptest.rs` | 234 | async=1, panic=5, hub=2 |
| `checkpoint/tests/time_travel_correctness.rs` | 854 | async=17, panic=61, hub=5 |
| `checkpoint/tests/txid_monotonicity_and_recovery.rs` | 305 | async=6, panic=17, hub=2 |
| `mvcc/src/lib.rs` | 113 | panic=1, alt_spec=2 |
| `mvcc/src/seq_log.rs` | 856 | panic=11 |
| `mvcc/src/snapshot.rs` | 963 | panic=4 |
| `mvcc/src/tombstone.rs` | 53 |  |
| `mvcc/tests/loom_sequence_log.rs` | 161 | panic=10 |
| `mvcc/tests/loom_snapshot_registry.rs` | 167 | panic=8 |
| `mvcc/tests/tombstone_diff.rs` | 41 | hub=1 |
| `mvcc/tests/visibility_proptest.rs` | 109 |  |
