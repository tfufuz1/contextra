# IMPL-02 — Rückbau: Löschkette und Forschungs-Heuristiken

**Phase:** P0 · **SOLL:** SPEC §1.4 (Nicht-Ziele), C-14, C-15, §20.3 · **Ergänzt:** `docs/LOESCHKETTE_ENTFERNUNG_SPEC.md` (bleibt gültig für die Löschkette; dieses Dokument korrigiert dessen Ausgangslage und fügt Teil B hinzu)

## 0. Korrektur der Ausgangslage

`LOESCHKETTE_ENTFERNUNG_SPEC.md` Welle 0/1 (Entfernen von `contextra-audit-export`, `contextra-avv-generator`) ist in HEAD **bereits erledigt** (Workspace-Mitglieder und Verzeichnisse fehlen). Offen sind Wellen 2–5. Gemessener Rest (Dateien mit Treffer):

| Crate | Dateien mit Löschketten-Symbolen |
|---|---:|
| `contextra-crypto` | 40 |
| `contextra-store` | 14 |
| `contextra-mcp` | 14 |
| `contextra-engine` | 6 |
| `contextra-privacy` | 6 |
| `contextra` | 6 |
| `contextra-db` | 4 |
| `contextra-graph` | 2 |
| `contextra-kvcache`, `-py`, `-types` | je 1 |

Reproduktion: `grep -rEl 'DeletionProof|LayerCleanupProof|kv_shredding|KeyRegistry|AuditChain|RevocationLog|deletion_proof|CryptoShred|GraphRepairAttestation' --include=*.rs --include=*.toml --include=*.py crates xtask benchmarks`

---

# Teil A — Löschkette

## A1. Zielbild (C-15)

`retire(gid)` = versionierter Tombstone-Commit · Retention + Compaction entfernt tote Zeilen · `purge` = Best-Effort-Override · Derivate per Referenzzählung · Betriebslogs per `tracing`. **Kein** Löschnachweis, keine Signatur, kein Crypto-Shredding, keine hash-verkettete AuditChain, kein `RevocationLog`.

Wichtig: Das **Verschlüsselungs-Fundament bleibt** (SPEC §8.3: AEAD, KEK/DEK, keyed BLAKE3). Entfernt wird nur, was *Löschung beweist* oder *per Schlüsselvernichtung erzwingt*. Die KEK/DEK-Hierarchie wird in IMPL-08 neu auf Pack-Ebene gebaut; `kv_shredding.rs` ist dafür **keine** Vorlage (Gruppen-/Record-DEK pro Zeile widerspricht „Pack-DEK, O(1)-KEK-Rotation").

## A2. Entscheidungsmatrix `contextra-crypto`

| Datei | Entscheidung | Begründung / Ziel |
|---|---|---|
| `deletion_proof.rs`, `deletion_proof_typestate.rs`, `ed25519_proof.rs` | **löschen** | Löschnachweise. `DeletionProofKeyPair`, `SignatureVersion` entfallen. **Ed25519 selbst** bleibt als Abhängigkeit für Commit-Signaturen (SPEC §4.3) — Wrapper neu in `contextra-security` (IMPL-08) |
| `audit_chain.rs` | **löschen** | Commit-Kette ersetzt sie (SPEC §8.4, Hash-verketteter DAG in `mvcc`) |
| `revocation_log.rs`, `kv_shredding.rs` | **löschen** | Crypto-Shredding |
| `wal_completeness.rs` | **behalten bis IMPL-03**, dann löschen | prüft HMAC-Kette der WAL; SOLL-WAL nutzt CRC32C + LSN (SPEC §12.1) |
| `wal_crypto.rs` (`EncryptedWal`, `WalHmac`, `IntegrityVerifier`) | **behalten bis IMPL-03**, Neuentscheidung dort | `OPEN-03-1` |
| `crypto.rs` (`KeyManager`) | **umbauen**: Methoden `derive_deletion_proof_key`, `create_deletion_proof`, `derive_segment_key*`, `derive_kv_key*` raus; `derive_file_key`, `integrity_key`, `encrypt/decrypt_auto_nonce`, `emergency_wipe` bleiben vorläufig | Vorarbeit für IMPL-08 |
| `kdf.rs` (Argon2id), `anti_tamper.rs` (`VolatileEncryptionKey`) | **behalten** | Fundament §8.3 |
| `kv_cipher.rs`, `kv_segment/*` | **verschieben** nach `contextra-kvcache` (Phase 5, optional) — dort existiert bereits eine Duplikat-Implementierung (`TenantIsolatedKvStore`, `EvictionWorker`, `KvSegment`) | `OPEN-02-1`: Duplikate konsolidieren, **eine** Implementierung behalten (kvcache) |
| `error.rs` | `CryptoError`-Varianten zu den entfernten Modulen löschen | — |

## A3. Weitere Crates (Symbolliste → Aktion)
