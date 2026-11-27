# Implementierungsspezifikation `contextra-crypto` (Löschkette)

*Ring 0 · Konzept-WP 1 und 11 · Abhängig von: 01, 02 · Blockiert: 03 (Wurzel, A-2), 10 §6 · Index: `00-INDEX.md` (K-7, K-25, K-33, K-40, K-41, A-2)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck und Abgrenzung

Drei Aufgaben: (1) **Rückbau** des Beweiscodes (`deletion_proof`, `ed25519_proof`) ohne den KV-Cache-Shredding-Pfad zu zerstören; (2) **Schlüsselwurzel** `ErasureRoot` als einzige Quelle für Ledger-MAC und Journal-Logschlüssel (behebt K-33 zusammen mit 03); (3) **Stufe 2** (G2): reine RAM-Logik `SaltVault`.
Keine I/O, keine Zeit, kein Zufall in `crypto/src` (Crate-Invariante ✔ `lib.rs`): Nonces, Salts und Wurzeln werden **hineingereicht**.

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Features | `default = ["legacy-erasure","kv-encryption","test-utils"]`; `kv-encryption = ["legacy-erasure"]` (`Cargo.toml:39-42`). Eine Required-Features-Zeile bei `Cargo.toml:60` (Ziel **[?]**) |
| Gating in `lib.rs` | `legacy-erasure` schaltet `deletion_proof`, `ed25519_proof`, `kv_shredding`, `revocation_log` und Re-Exporte `DeletionProof*`, `KvSegmentCipher`, `KeyRegistry`, `RevocationLog` (`lib.rs:47-99`). Weitere `cfg` in `audit_chain.rs:26,29,175` und `kv_cipher.rs:36,107,114,120,220` |
| Außerhalb von `crypto` | Nutzer der Legacy-Symbole: `contextra/tests/collection_profile_matrix.rs`, `contextra/tests/deletion_e2e.rs`, `store/tests/campaign_j42_audit.rs` (K-25: `DeletionProofKeyPair`) |
| `dek.rs` | `UnitKey`, `Dek`, `AncestorSalts{t,c,s:Option,d}`, `derive_dek` verwirft `hkdf_expand` mit `let _ =` (`dek.rs:76`, K-40), `wrap_dek(kek,dek,nonce)`, `unwrap_dek`. Konstanten `DEK_LABEL`, `DEK_SALT_LABEL`, `WRAP_AAD` |
| `kdf.rs` | `hkdf_expand(prk, info, out)`, `ledger_mac(key, bytes)`, `derive_key_argon2id`, `KdfHeader` |
| Store-Nutzung | `StoreErasureKernel::open_early` ruft `contextra_crypto::hkdf_expand` mit Labels `ctx-ledger-mac-v1`, `ctx-erasure-log-v1` auf einem `prk` aus `KeyMaterialSource::fill_key_bytes` (`erasure/mod.rs:337-346`); `recovery.rs:228` übergibt `OsKeyMaterialSource` ⇒ **Zufall pro Start** (K-33) |

## 3. Rückbau (WP-1)

### 3.1 Feature-Schnitt (K-7/K-41)

```toml
[features]
default       = ["kv-shredding", "kv-encryption", "test-utils"]
kv-shredding  = []                    # KeyRegistry, RevocationLog, KvSegmentCipher, EncryptedCommitmentSalt, derive_subkey
kv-encryption = ["kv-shredding"]
legacy-proof  = []                    # deletion_proof, ed25519_proof; NICHT im Default; nur zum Auslaufen
# legacy-erasure: entfällt
```

Regel: **Alles, was der KV-Cache-Shredding-Pfad braucht, wechselt von `legacy-erasure` auf `kv-shredding`**; nur `deletion_proof/*`, `ed25519_proof.rs` und deren Re-Exporte (`CustodyLevel`, `DeletionLayer`, `DeletionProof`, `DeletionProofKeyPair`, `DeletionScope`, `ExcludedScope`, `LayerCleanupProof`, `SigningKeyHandle`, `DeletionProofError`, `SignatureVersion`) wechseln auf `legacy-proof`.

| Datei | Aktion |
| --- | --- |
| `Cargo.toml` | Features wie oben; Zeile 60 (`required-features`) auf `kv-shredding` oder `legacy-proof` je nach Ziel umstellen **[?]** |
| `lib.rs:47-99` | `cfg`-Attribute nach Regel umstellen |
| `audit_chain.rs:26,29,175` | `EncryptedCommitmentSalt` und Verwandte → `kv-shredding`; Teile, die `DeletionProof` referenzieren → `legacy-proof` |
| `kv_cipher.rs` | `KvSegmentCipher` (`:36,107,114,120,220`) → `kv-shredding` |
| Tests `crypto/tests/*` mit `#![cfg(feature = "legacy-erasure")]` | `deletion_proof_*`, `ed25519_proof_external_verify.rs`, `campaign_cov_deletion_proof.rs`, `campaign_j26_hypotheses.rs`, `guarantee_deletion_irrecoverable.rs`, `crypto_crossimpl_python.rs` → `legacy-proof` bzw. löschen; `kv_segment_integration.rs`, `revocation_log*.rs`, `campaign_cov_revocation_log.rs`, `audit_chain_tests.rs`, `guarantee_audit_chain_tamper.rs` → `kv-shredding`. Zuordnung pro Datei durch Lesen der importierten Symbole (`rg "use contextra_crypto" <datei>`) |
| `store/tests/campaign_j42_audit.rs` | `DeletionProofKeyPair`-Nutzung entfernen (K-25) |
| `contextra/tests/{deletion_e2e,collection_profile_matrix}.rs` | Legacy-Symbole entfernen; `deletion_e2e.rs` wird in 13 §7 zum Canary `T-DEL-1` |

Anschließend: `rg "legacy-erasure|legacy_erasure" .` ⇒ 0 Treffer außerhalb `docs/`. `cargo build --workspace --locked` mit `--no-default-features` je Crate prüfen **[?]** (Features anderer Crates, die `contextra-crypto` mit `default-features = false` einbinden).

### 3.2 `derive_dek` fallibel (K-40)

```rust
pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Result<Dek>;  // Signatur ändert sich
```
`hkdf_expand(...)?`; Fehler als `CryptoError::Kdf` (Variante vorhanden **[?]** `error.rs`). Die Funktion hat laut B-14 keine Nutzer außerhalb `crypto` (Prüfauftrag `rg "derive_dek" crates`); Nutzer in Tests werden angepasst. CI-Gate G-2 (`let _ =`) erfasst die Datei künftig (neue Erase-Pfad-Liste enthält `crypto/src/dek.rs`, `crypto/src/vault.rs`, `crypto/src/erasure_root.rs`).

## 4. `ErasureRoot` (A-2, behebt K-33)

Neue Datei `crypto/src/erasure_root.rs` (kein I/O, nur Ableitung):

```rust
/// 32 Byte Wurzelmaterial. Zeroize on drop. Kein Debug-Inhalt, kein Clone, kein Serialize.
pub struct ErasureRoot(SecretKey<32>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum ErasureLabel { LedgerMac, ErasureLog, VaultEpoch, JobKey }

impl ErasureLabel {
    /// Eingefroren (Golden-Test): bestehende Labels bleiben bytegleich.
    pub const fn info(self) -> &'static [u8] {
        match self {
            Self::LedgerMac  => b"ctx-ledger-mac-v1",
            Self::ErasureLog => b"ctx-erasure-log-v1",
            Self::VaultEpoch => b"ctx-vault-epoch-v1",
            Self::JobKey     => b"ctx-erasure-job-v1",
        }
    }
}

impl ErasureRoot {
    pub fn from_bytes(b: [u8; 32]) -> Self;
    /// HKDF-SHA256 (hkdf_expand) mit info = label.info() ‖ context. Fallibel, kein `let _ =`.
    pub fn derive(&self, label: ErasureLabel, context: &[u8]) -> Result<SecretKey<32>>;
}
```

Regeln: Derselbe Root + Label + Kontext ⇒ **identischer** Schlüssel (Test TST-CRYPTO-ROOT-1). Verschiedene Labels ⇒ verschiedene Schlüssel (-2). `JobKey` bekommt als Kontext die 16 Byte `ErasureId` (je Auftrag eigener Schlüssel). Der Store ruft **ausschließlich** `ErasureRoot::derive` (kein direktes `hkdf_expand` mehr in `store/src/erasure/mod.rs:337-346`). Bereitstellung der Wurzel: 03 (A-2: `ErasureRootProvider`, `FileErasureRoot`, `DerivedErasureRoot`, `StaticErasureRoot`). `DerivedErasureRoot` nutzt `KeyManager` + `ErasureLabel`-Ableitung (`ErasureRoot` aus `KeyManager::derive_subkey`-ähnlicher API **[?]** `crypto.rs` auf passende Methode prüfen; Fallback: `hkdf_expand(master, b"ctx-erasure-root-v1")`).

## 5. Stufe 2: `SaltVault` (WP-11, opt-in)

Hierarchie (Konzept 5.9): `ErasureRoot` →`UnitKey` je Mandant → `AncestorSalts{t,c,s,d}` → `derive_dek(unit, salts, doc_tag)` → AES-256-GCM-SIV.

### 5.1 Typen (`crypto/src/vault.rs`, rein RAM)

```rust
pub struct VaultGeneration(pub u64);                       // streng monoton
pub enum SaltLevel { Tenant, Collection, Subject, Document }
pub struct SaltPath { pub tenant: u64, pub collection: Option<u64>, pub subject: Option<[u8;16]>, pub doc: Option<u64> }

pub struct SaltVault { /* BTreeMap<SaltPath, Salt>, generation, epoch_key: SecretKey<32> */ }
impl SaltVault {
    pub fn new(epoch_key: SecretKey<32>) -> Self;
    /// Salt zum Pfad; fehlt er ⇒ aus `fresh` (vom Aufrufer, CSPRNG) angelegt. Gibt None, wenn der Pfad vernichtet ist (Tombstone).
    pub fn salts_for(&mut self, path: &SaltPath, fresh: impl FnMut() -> Result<Salt>) -> Result<Option<AncestorSalts>>;
    /// Vernichtet den Salt auf `level` unter `path` und markiert alle Nachfahren als vernichtet. Idempotent.
    pub fn destroy(&mut self, path: &SaltPath, level: SaltLevel) -> Result<DestroyReceipt>;
    /// Serialisiert (versioniert, Magic `CTXV`), jeder Eintrag einzeln mit `wrap_dek`-artiger AEAD (AAD = Pfad, Generation) unter `epoch_key`.
    pub fn seal_snapshot(&self, nonce_for: impl FnMut() -> Result<[u8;12]>) -> Result<Vec<u8>>;
    pub fn open_snapshot(epoch_key: SecretKey<32>, bytes: &[u8]) -> Result<Self>;
    /// Neue Generation: verbleibende Salts mit neuem Epoch-Key umhüllt; alter Key wird zeroisiert (Rückgabe: Snapshot-Bytes).
    pub fn rotate(&mut self, new_epoch: SecretKey<32>, nonce_for: impl FnMut() -> Result<[u8;12]>) -> Result<Vec<u8>>;
}
```

`destroy` entfernt den Salt-Wert **und** lässt keinen Hinweis auf den Wert zurück (nur Pfad + `destroyed`-Marker, kein Hash des Salts). Nonce-Disziplin: AES-GCM-SIV ist nonce-misuse-resistent, trotzdem erzeugt der Aufrufer (Store, `KeyMaterialSource`) frische 12 Byte je Eintrag; `crypto` prüft keine Eindeutigkeit, **Test** (-6) prüft, dass `seal_snapshot` bei zweimal derselben Nonce-Folge in verschiedenen Generationen unterschiedliche Chiffre liefert (AAD enthält Generation).

### 5.2 Vernichtung (Protokoll, Zuständigkeit verteilt)

1. Treiber (10): Zustand `KeysDestroying` (nur wenn `profile = Regulated` oder `stage2` aktiv).
2. Store (`vault_store.rs`, 03 WP-11): `destroy` im RAM → `rotate` → `write_atomic(vault.<gen>.ctxv)` → `sync_dir` → alte Generation entfernen (Pending-Delete) → Journal `KeysDestroyed`. **Erst danach** gilt G2.
3. Bündelung: mehrere Aufträge im Koaleszenzfenster ⇒ **eine** Rotation (jede kostet O(Vault)).
4. Crash: Vor `write_atomic` unverändert; nach `write_atomic` vor Journal: Boot erkennt höhere Generation ⇒ `KeysDestroyed` nachtragen (idempotent).

### 5.3 Versiegelung (Format, 01/03 stellen Hooks)

`RecordSealer` aus `ports` (nach `seal.rs` verschoben, Feature `stage2`) wird in `crypto` implementiert: `AES-256-GCM-SIV`, AAD = `tenant ‖ collection ‖ BLAKE3(key) ‖ record_type u8 ‖ format_version u8`. Der Store versiegelt `StoredDocument`, WAL-Payload, `fw:`, Graph-Entity-Payload, `CheckpointMeta.metadata`. **Nicht versiegelbar:** Postings, Term-Dictionary, HNSW-Vektoren, CSR-Struktur ⇒ G3 Pflicht. Dual-Read, seal-on-rewrite, Backfill: Spezifikation in 03 (WP-11); `crypto` liefert nur `seal(ctx, plaintext, nonce)` / `open(ctx, bytes)`.

## 6. Tests

| ID | Inhalt |
| --- | --- |
| TST-CRYPTO-FEAT-1 | `cargo build -p contextra-crypto --no-default-features --features kv-shredding` und `--features legacy-proof` bauen; `--no-default-features` allein baut |
| TST-CRYPTO-FEAT-2 | KV-Shredding-Regression: `kv_segment_integration`, `revocation_log_vfs` laufen grün unter `kv-shredding` |
| TST-CRYPTO-DEK-1 | `derive_dek` liefert `Err`, wenn `hkdf_expand` fehlschlägt (Fehlerinjektion über übergroße `out`-Länge ist nicht möglich ⇒ Test über Hilfsfunktion `derive_dek_with(expand: impl Fn…)`) |
| TST-CRYPTO-ROOT-1…3 | Determinismus, Label-Trennung, Golden-Vektoren für `LedgerMac`/`ErasureLog` (Bytes eingefroren) |
| TST-CRYPTO-VAULT-1…7 | `destroy` idempotent; Nachfahren unableitbar (`salts_for` ⇒ `None`); Snapshot-Roundtrip; falscher Epoch-Key ⇒ `Err`; Manipulation (Bitflip) ⇒ `Err`; `rotate` macht alten Snapshot mit altem Key weiter lesbar, **aber** der vernichtete Salt fehlt in neuer Generation; Zeroize-Test (`ZeroizeOnDrop`) |
| TST-CRYPTO-SEAL-1 | Roundtrip, AAD-Bindung (anderer Schlüssel ⇒ `Err`) |
| TST-CRYPTO-NOIO | `rg "std::fs|tokio::fs|SystemTime|Instant::now|OsRng" crypto/src/vault.rs crypto/src/erasure_root.rs` ⇒ leer |

## 7. Arbeitspakete

| WP | Inhalt | Verdrahtung (kein `dead_code`-Problem in `crypto`, aber Nutzer fordern Nachweis) |
| --- | --- | --- |
| 5.1 | Feature-Schnitt, Tests umhängen | Workspace baut |
| 5.2 | `derive_dek` fallibel | – |
| 5.3 | `ErasureRoot` + Tests | wird in 03 (A-2) sofort verdrahtet |
| 5.4 | `SaltVault`, `RecordSealer`-Impl | erst mit WP-11 (hinter `stage2`) |

## 8. Definition of Done

Keine Treffer für `legacy-erasure`; KV-Shredding-Tests grün; `crypto/src` ohne I/O/Zeit/Zufall (Gate G-5 erweitert um diese Dateien); jede neue öffentliche Funktion mit „# Fehler / # Crash“-Doku.

## 9. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-C1 | Ziel von `Cargo.toml:60` `required-features` | Lesen, dann Feature zuordnen |
| O-C2 | Wie liefert `KeyManager` die Basis für `DerivedErasureRoot`? | Eigene Methode `KeyManager::derive_erasure_root(&self) -> Result<ErasureRoot>` mit festem Label |
| O-C3 | `CryptoError`-Variante für KDF-Fehler | vorhandene nutzen, sonst am Ende anhängen |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `src/erasure_root.rs` (74 LOC) ✔: `ErasureRoot` (zeroizing, nicht klonbar), `ErasureLabel{LedgerMac, ErasureLog, VaultEpoch, JobKey}`, `derive(label, context) -> Result<SecretKey<32>>`. **K-40 behoben:** `derive_dek` propagiert den HKDF-Fehler (`dek.rs:76`). **Offen:** Der Store nutzt `ErasureRoot` nicht (K-33, K-60); Feature `kv-shredding` und Rückbau von `legacy-erasure` (K-7/K-41) sind nicht umgesetzt (`Cargo.toml:39-42` unverändert).

**Verbindliche Ergänzungen.**

1. **Verdrahtung.** Der Store ist der einzige Verwender von `ErasureRoot` für `LedgerMac`/`ErasureLog`. `JobKey` (Kontext = `ErasureId`) und `VaultEpoch` sind für Stufe 2 reserviert und dürfen nicht anderweitig benutzt werden. HKDF-Labels bleiben eingefroren (Golden-Test über die vier `info()`-Werte).
2. **Wurzelquelle** (A-2): `FileErasureRoot` | `DerivedErasureRoot` | `StaticErasureRoot`. Die Zufallsquelle für die **Erst**-Erzeugung ist `KeyMaterialSource` (OS-CSPRNG, fail-closed). Der Wurzel-Dateipfad liegt in `erasure_dir`, nicht im Datenverzeichnis des LSM (Backup-Trennung), Modus 0600, `write_atomic` + `sync_dir` (CP-27).
3. **Testquellen.** `StaticKeyMaterialSource` füllt bei erschöpftem Strom ein deterministisches Muster (`key_source.rs`) und ist `pub` exportiert. Für Regulated gilt: Start verweigert, wenn die Quelle nicht `OsKeyMaterialSource` ist **oder** die Wurzel nicht extern ist. Zusätzlich `cfg(any(test, feature = "test-utils"))` für `StaticKeyMaterialSource` prüfen **[?]** (`TST-CRYPTO-STATIC-1`).
4. **Nachweis der Unwiderruflichkeit (Stufe 2).** `KeysDestroying` darf erst `Purging` freigeben, wenn die Schlüsselvernichtung `sync`-durabel ist und ein Wiederlesen der Wurzel/`VaultEpoch` fehlschlägt (`TST-CRYPTO-SHRED-1`). Medienreste (SSD, Snapshots) sind nur über Stufe 2 abgedeckt (14 §4 Zeile 20).
5. **Zeroize.** `SecretKey` zeroisiert beim Drop; `ErasureRoot` hat kein `Debug`-Inhalt/`Clone`/`Serialize` (Compile-fail-Test `TST-CRYPTO-ROOT-1`).
