# Extraktion fuer `acta-ledger`

**Aufgabe:** Append-only-Ledger K/E/A, Checkpoints, Commitments, RegistrySink, DirSink  
**Level:** L2 · **Budget:** 2800 LOC · **Spezifikation:** docs/SPEC.md Kap. 6.2  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs  
**Umfang des Archivs:** 10 Dateien, 1855 Zeilen (inkl. Tests)  

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
| `erasure/src/ledger.rs` | 210 |  |
| `erasure/src/ledger/durable.rs` | 291 | alt_spec=4 |
| `erasure/src/ledger/entry.rs` | 77 | serde=1 |
| `erasure/src/ledger/error.rs` | 49 |  |
| `erasure/src/ledger/persistent.rs` | 185 | serde=2, panic=4, alt_spec=3 |
| `erasure/src/ledger/record.rs` | 318 |  |
| `erasure/src/ledger/sink.rs` | 229 | alt_spec=6 |
| `erasure/src/ledger/storage.rs` | 185 | std_fs=9 |
| `ports/src/ledger.rs` | 180 | serde=1 |
| `ports/src/report_signer.rs` | 131 | panic=1, alt_spec=1 |
