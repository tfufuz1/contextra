# Extraktion fuer `acta-vault`

**Aufgabe:** Mandantentresore, KeyAnchor, FileAnchor, Wurzelrotation, Sicherungsexport  
**Level:** L2 · **Budget:** 5500 LOC · **Spezifikation:** docs/SPEC.md Kap. 4.3, 4.4  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs  
**Umfang des Archivs:** 21 Dateien, 6797 Zeilen (inkl. Tests)  

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
| `erasure/src/kv_shredding.rs` | 980 | serde=1 |
| `erasure/src/revocation_log.rs` | 466 | panic=5, serde=6 |
| `erasure/src/vault.rs` | 383 | float=1 |
| `erasure/src/vault/durable.rs` | 651 | float=5 |
| `erasure/src/vault/key_store.rs` | 215 | serde=1 |
| `erasure/src/vault/keyring.rs` | 158 | async=5 |
| `erasure/src/vault/open.rs` | 236 |  |
| `erasure/src/vault/rehash.rs` | 169 | float=4 |
| `erasure/src/vault/scrub.rs` | 130 |  |
| `erasure/src/vault/sealer.rs` | 825 | async=7, serde=1 |
| `erasure/src/vault/slot.rs` | 431 |  |
| `erasure/src/vault/table.rs` | 375 | float=5 |
| `erasure/tests/p1_key_hierarchy_and_purge.rs` | 243 | panic=19 |
| `erasure/tests/p2_vault_rewrite_tests.rs` | 346 | panic=9, async=8, alt_spec=2 |
| `erasure/tests/shredding_envelope_test.rs` | 118 | panic=5 |
| `erasure/tests/t_sealer_keyring.rs` | 58 | panic=6, async=5 |
| `erasure/tests/t_vault_core.rs` | 114 | panic=8 |
| `erasure/tests/vault_slot_layout.rs` | 91 | panic=2, alt_spec=1 |
| `erasure/tests/vault_spec_conformance.rs` | 567 | panic=29, async=3 |
| `ports/src/key_material.rs` | 37 |  |
| `ports/src/keys.rs` | 204 | serde=1, unsafe=1 |
