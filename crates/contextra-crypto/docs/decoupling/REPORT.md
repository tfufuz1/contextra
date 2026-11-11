# Abschlussbericht Auftrag S-38: contextra-crypto

## 1. Gate-Matrix
- **G-A (Kernvollständigkeit):**
  - Befehl: `cargo check -p contextra-crypto --no-default-features --locked`
  - Ergebnis: GRÜN (0 Fehler).
  - Befehl: `cargo test -p contextra-crypto --no-default-features --test core_smoke --locked`
  - Ergebnis: GRÜN (1 passed, 0 failed).
- **G-B (Feature-Matrix):**
  - Befehl: `cargo check -p contextra-crypto --no-default-features --features <f> --locked` für alle Features (`kv-encryption`, `test-utils`, `cloud-egress-guard`, `workspace`).
  - Ergebnis: GRÜN.
- **G-C (Qualität):**
  - Befehl: `cargo fmt -p contextra-crypto -- --check`
  - Ergebnis: GRÜN.
  - Loom-Tests:
    - `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-crypto --test loom_kv_deferred_zeroize` -> GRÜN.
    - `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-crypto --test loom_eviction_worker_shutdown_race` -> GRÜN.
- **G-D (Null-Prüfung):**
  - Befehl: `grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-crypto/src --include=*.rs | grep -v '/compat/'`
  - Ergebnis: LEER (0 Treffer).
  - Befehl: `python3 tools/count_bridge_refs.py -c contextra-crypto`
  - Ergebnis: 0.
- **G-E (Layering und Isolation):**
  - Befehl: `cargo tree -p contextra-crypto -e normal --no-default-features`
  - Ergebnis: Keine unerlaubten internen Abhängigkeiten.
- **G-F (Standalone):**
  - Befehl: `cargo build -p contextra-crypto --no-default-features --example standalone --locked` -> GRÜN.
  - Befehl: `cargo test -p contextra-crypto --no-default-features --doc --locked` -> GRÜN.
- **G-G (Konsumentenprobe):**
  - Befehl: `cargo check --workspace --lib --bins --locked`
  - Ergebnis: GRÜN nach jedem Commit.
- **G-H (Disjunktheit):**
  - Befehl: `git diff --stat origin/main...HEAD`
  - Ergebnis: Ausschließlich Dateien unter `crates/contextra-crypto/**` und `crates/contextra-kvcache/**`.
- **G-I (Formate):**
  - Befehl: `cargo test -p contextra-crypto --no-default-features --test golden_corpus --locked`
  - Ergebnis: GRÜN (3 passed, 1 ignored; alle Prüfungen gegen `MANIFEST.sha256` erfolgreich).

## 2. Abweichungen von Prompt und Spec
- Die eingecheckte Golden-Datei `dek_wrapping.bin` enthielt fälschlicherweise eine Escaped-String-Repräsentation statt roher Bytes; wurde repariert, sodass die SHA-256-Prüfung exakt übereinstimmt.

## 3. Getroffene Entscheidungen
- `KeyManager` verfügt additiv über `[u8; 32]`-Signaturen (`derive_kv_key_bytes`, `derive_kv_key_scoped_bytes`, `cipher_for_bytes`, `cipher_for_scoped_bytes`), um ohne `ModelFingerprint`-Kopplung genutzt werden zu können.
- KV-Code verbleibt in `contextra-crypto` unverändert additiv markiert (`// MIGRATION(S-38): Besitzer wird contextra-kvcache`), um Fremdnutzer (`contextra-infer-candle`, `contextra-store`) nicht zu brechen.

## 4. Offene Punkte und Risiken
- Keine offenen Punkte für `contextra-crypto`.

## 5. Übergaben an den Orchestrator
- **Löschung des KV-Codes in contextra-crypto:** Sobald alle Fremdnutzer migriert sind.
- **Fremdnutzer-Inventar:**
  - `crates/contextra-infer-candle/src/kv_bridge.rs:13`: `EncryptedKvLayer`, `KvSegment`, `KvSegmentCipher`, `TenantIsolatedKvStore`, `CryptoKey`, `EvictionWorker`.
  - `crates/contextra-infer-candle/benches/kv_bridge_bench.rs:8`: `CryptoKey`, `KvSegmentCipher`, `TenantIsolatedKvStore`.
  - `crates/contextra-infer-candle/tests/kv_bridge_*.rs`: `CryptoKey`, `KvSegment`, `KvSegmentCipher`, `TenantIsolatedKvStore`.
  - `crates/contextra-store/src/wal/encode.rs:11`: `WalHmac`.
  - `crates/contextra-store/src/wal/replay.rs:10`: `IntegrityVerifier`, `WalEntrySnapshot`.
  - `crates/contextra-store/src/wal/io.rs:5`: `IntegrityVerifier`, `WalEntrySnapshot`.
  - `crates/contextra-store/src/wal/mod.rs:343`: `KeyManager`.
  - `crates/contextra-store/src/lsm/recovery.rs:373`: `verify_wal_chain_completeness`.
  - `crates/contextra-mcp/src/sandbox.rs:234`: `CryptoKey`.
- **Default-Flip für Z:** `default = ["kv-encryption", "workspace"]` bleibt vorerst aktiv, da `contextra-store` und `contextra-infer-candle` darauf angewiesen sind.

## 6. Messwerte
- Fehler ohne Default-Features vorher: 0
- Fehler ohne Default-Features nachher: 0
- Restzahl der Null-Prüfung: 0
