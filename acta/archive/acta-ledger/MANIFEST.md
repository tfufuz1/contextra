# Extraktion fuer `acta-ledger`

**Aufgabe:** Append-only-Ledger K/E/A, Checkpoints, Commitments, RegistrySink, DirSink  
**Level:** L2 · **Budget:** 2800 LOC · **Spezifikation:** docs/SPEC.md Kap. 6.2  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs  
**Umfang des Archivs:** 16 Dateien, 3268 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `erasure` | K/E/A-Ledger (6.2); DirSink = ledger/sink.rs |
| `ports` | Port-Vorlagen |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| serde | 4 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| panic | 5 | unwrap/expect/panic (G-LINT) |
| std_fs | 9 | std::fs direkt (G-FS: nur acta-fs) |
| alt_spec | 14 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `bincode`, `blake3`, `futures-util`, `parking_lot` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-ledger/`) | LOC | Befunde |
|---|---:|---|
| `erasure/src/ledger.rs` | 209 |  |
| `erasure/src/ledger/durable.rs` | 290 |  |
| `erasure/src/ledger/entry.rs` | 76 | serde=1 |
| `erasure/src/ledger/error.rs` | 48 |  |
| `erasure/src/ledger/persistent.rs` | 184 | panic=4, serde=2 |
| `erasure/src/ledger/record.rs` | 317 |  |
| `erasure/src/ledger/sink.rs` | 228 |  |
| `erasure/src/ledger/storage.rs` | 184 | std_fs=3 |
| `erasure/tests/ledger_spec_tests.rs` | 167 | panic=10, async=1 |
| `erasure/tests/p5_wal_sink_tests.rs` | 233 | panic=10, alt_spec=1 |
| `erasure/tests/p6_ledger_durability.rs` | 406 | panic=24, alt_spec=3 |
| `erasure/tests/p7_report_chain_tests.rs` | 279 | panic=15, std_fs=2 |
| `erasure/tests/revocation_log_vfs.rs` | 262 | panic=17 |
| `erasure/tests/t_report_ledger.rs` | 76 | panic=4 |
| `ports/src/ledger.rs` | 179 | serde=1 |
| `ports/src/report_signer.rs` | 130 | panic=1, async=1 |
