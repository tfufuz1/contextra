# Extraktion fuer `acta-vault`

**Aufgabe:** Mandantentresore, KeyAnchor, FileAnchor, Wurzelrotation, Sicherungsexport  
**Level:** L2 · **Budget:** 5500 LOC · **Spezifikation:** docs/SPEC.md Kap. 4.3, 4.4  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs  
**Umfang des Archivs:** 14 Dateien, 5274 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `erasure` | Tresor (4.3): Eintragsformat auf 92 B/AAD mit slot umstellen; Hash-Shards nur RAM-Index |
| `ports` | KeyAnchor-Vorlagen (4.4) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 15 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 10 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 2 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 1 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 16 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 5 | unwrap/expect/panic (G-LINT) |
| std_fs | 3 | std::fs direkt (G-FS: nur acta-fs) |
| hub | 1 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 83 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `aes-gcm-siv`, `ahash`, `bincode`, `blake3`, `futures-util`, `parking_lot`, `rand` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-vault/`) | LOC | Befunde |
|---|---:|---|
| `erasure/src/kv_shredding.rs` | 981 | serde=1, zeit_zufall=15 |
| `erasure/src/revocation_log.rs` | 467 | serde=6, zeit_zufall=1, panic=5 |
| `erasure/src/vault.rs` | 384 | float=1, alt_spec=14 |
| `erasure/src/vault/durable.rs` | 652 | float=5, std_fs=3, alt_spec=15 |
| `erasure/src/vault/key_store.rs` | 216 | serde=1 |
| `erasure/src/vault/keyring.rs` | 159 | alt_spec=8 |
| `erasure/src/vault/open.rs` | 237 | alt_spec=5 |
| `erasure/src/vault/rehash.rs` | 170 | float=4, alt_spec=5 |
| `erasure/src/vault/scrub.rs` | 131 | alt_spec=2 |
| `erasure/src/vault/sealer.rs` | 826 | serde=1, async=2, alt_spec=3 |
| `erasure/src/vault/slot.rs` | 432 | alt_spec=11 |
| `erasure/src/vault/table.rs` | 376 | float=5, alt_spec=17 |
| `ports/src/key_material.rs` | 38 | hub=1 |
| `ports/src/keys.rs` | 205 | serde=1, unsafe=1, alt_spec=3 |
