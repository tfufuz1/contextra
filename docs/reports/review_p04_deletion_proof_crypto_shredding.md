# Review: P04 — DeletionProof & Crypto-Shredding Korrektheitsprüfung

## 1. Zusammenfassung

Die kryptographische Schutzschicht in `crates/contextra-crypto` implementiert Löschnachweise (`DeletionProof` v1, v2, v3), Schlüsselverwaltung (`KeyManager`), Envelope-Crypto-Shredding (`KeyRegistry`) sowie ein Ed25519-signiertes Widerrufslog (`RevocationLog`). Die Verifizierung zeigt, dass die Version-3-Ed25519-Löschnachweise und die Längenpräfix-Hashes (`hash_deleted_keys_length_prefixed`) mathematisch robust und extern verifizierbar sind. Allerdings bestehen vier schwerwiegende Sicherheitslücken:
1. Löschen oder Fehlen der `revocation.log`-Datei auf Disk führt beim Neustart zu einem leeren Widerrufsregister, wodurch vormals gekündigte/geshreddete Schlüssel wieder zum Entschlüsseln freigegeben werden (**CRITICAL**).
2. Der Zeitstempel `timestamp` ist in Version 2 HMAC-Löschbeweisen nicht im Signatur-Payload enthalten und kann fälschungssicher manipuliert werden (**HIGH**).
3. Bei HMAC v1/v2-Beweisen wird der HMAC-Schlüssel direkt vom Datenbank-Master-Key abgeleitet, sodass ein Angreifer mit Dateisystemzugriff rückwirkend valide Löschnachweise fälschen kann (**HIGH**).
4. `DeletionProofKeyPair` versäumt die RAM-Zeroization (`ZeroizeOnDrop`) des Ed25519-Signierschlüssels (**HIGH**).

---

## 2. Geprüfte Dateien

- `docs/reports/deletion_proof_verifiability_report.md`: Doku-Report zu Abnahmekriterien AK-20/AK-21 und Systemgrenzen (Ausgangshypothese).
- `crates/contextra-crypto/README.md`: Öffentliche API-Spezifikation und Sicherheitsinvarianten von Ring 0.
- `crates/contextra-crypto/src/deletion_proof.rs`: Datenstrukturen und Verifikationslogik für `DeletionProof` (v1, v2, v3) und `LayerCleanupProof`.
- `crates/contextra-crypto/src/ed25519_proof.rs`: Standalone Ed25519-Signierung, Typisierung (`SignatureVersion`) und externe Verifikation.
- `crates/contextra-crypto/src/kv_shredding.rs`: `KeyRegistry` für KEK/DEK Envelope-Encryption und O(1) Crypto-Shredding.
- `crates/contextra-crypto/src/revocation_log.rs`: Persistiertes, Ed25519-signiertes Append-Only Widerrufslog.
- `crates/contextra-crypto/src/crypto.rs`: `KeyManager` für AES-256-GCM-SIV, HKDF-Schlüsselableitung und Nonce-Generierung.
- `crates/contextra-crypto/src/wal_crypto.rs`: `EncryptedWal`, `WalHmac` und `IntegrityVerifier` für WAL-Hashketten.
- `crates/contextra-crypto/src/anti_tamper.rs`: `VolatileEncryptionKey` mit Cold-Boot-Schutz und Zeroization.
- `crates/contextra-crypto/src/kv_segment/segment.rs`: `KvSegment` mit `ZeroizeOnDrop`-Garantie für In-Memory-Tensoren.
- `crates/contextra-crypto/src/kv_cipher.rs`: High-Level `KvSegmentCipher` zur Anbindung an Engine/Cache.

---

## 3. Extrahierte Invarianten

- **I-1 (Physische Bereinigungsreihenfolge / INV-DELETION-1):** `DeletionProof::create` / `create_v3` darf NUR nach nachgewiesener physischer Layer-Bereinigung erzeugt werden. `LayerCleanupProof::new_after_verified_empty` erzwingt `remaining_live_entries == 0`.
- **I-2 (Externe Asymmetrische Verifizierbarkeit v3):** `DeletionProof::verify_external` verifiziert v3-Beweise rein asymmetrisch mittels Ed25519 `VerifyingKey` (32 Bytes) ohne Zugriff auf den Master-Schlüssel oder internen Datenbank-Zustand.
- **I-3 (Kollisionsfreie Schlüsselraum-Kodierung):** `hash_deleted_keys_length_prefixed` stellt jedem Schlüssel im Blake3-Byte-Strom ein 4-Byte Little-Endian Längenpräfix voran, um Schlüsselteilungs-Kollisionen (z. B. `["ab", "c"]` vs `["a", "bc"]`) unmöglich zu machen.
- **I-4 (O(1) Crypto-Shredding & Schlüsselvernichtung):** `KeyRegistry::revoke_group` / `revoke_record` vernichtet den KEK bzw. DEK-Wrap im Speicher in O(1) Zeit und macht zugehörige Ciphertexts unwiderruflich unlesbar.
- **I-5 (Manipulationssichere Widerrufs-Persistenz):** `RevocationLog` führt eine Ed25519-signierte, SHA-256-hash-verkettete Append-Only-Liste aller Widerrufe mit atomarem Temp-File-Swap (`rename`).
- **I-6 (In-Memory Zeroization / Cold-Boot-Schutz):** Flüchtige Schlüsselstrukturen (`VolatileEncryptionKey`, `SubKey`, `GroupKek`, `RecordDek`, `WalEntrySnapshot`, `KvSegment`) überschreiben Schlüssel- und Sensordaten bei `Drop` explizit im RAM mit Nullen (`Zeroize` / `ZeroizeOnDrop`).
- **I-7 (Domain Separation & Monotone Nonce-Unikatsgarantie):** `KeyManager` erzeugt 12-Byte-Nonces aus 4-Byte Zufall und 8-Byte atomarem Sequenzzähler; HKDF-Expansionen nutzen strikte Domain-Separations-Präfixe.
- **I-8 (Constant-Time Vergleich bei symmetrischer Verifikation):** Checksummen- und HMAC-Vergleiche in v1/v2-Beweisen und WAL-Quittungen nutzen `subtle::ConstantTimeEq`, um Timing-Side-Channels zu unterbinden.

---

## 4. Befunde

### [CRITICAL] F-3 — Fehlen oder Löschung der `RevocationLog`-Datei hebelt Crypto-Shredding nach Systemneustart lautlos aus

- **Ort:** `crates/contextra-crypto/src/revocation_log.rs:133-176` (`RevocationLog::open_or_create`), `crates/contextra-crypto/src/kv_shredding.rs:188-202` (`KeyRegistry::is_group_revoked`)
- **Invariante betroffen:** I-4, I-5
- **Beleg:**
```rust
// crates/contextra-crypto/src/revocation_log.rs:138-158
if path_buf.exists() {
    let mut file = File::open(&path_buf).map_err(...)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).map_err(...)?;
    if !contents.is_empty() {
        let parsed_entries: Vec<RevocationEntry> = bincode::deserialize(&contents)...;
        Self::verify_chain_entries(&parsed_entries, &verifying_key)?;
        for entry in &parsed_entries {
            revoked_targets.insert(entry.target.clone());
        }
        entries = parsed_entries;
    }
}
```
- **Angriffs-/Fehlerszenario:**
  1. Ein Mandant veranlasst das Crypto-Shredding von `group_id = 42`.
  2. `KeyRegistry::revoke_group(42)` führt `log.append(RevocationTarget::Group(42))` aus, fügt die Revocation in das `revoked_targets`-Set ein und vernichtet den KEK im RAM.
  3. Der Prozess wird beendet oder startet neu. Vor oder während des Starts löscht oder verschiebt ein Angreifer mit Dateisystemzugriff (oder ein fehlerhaftes Deployment-Skript) die Datei `revocation.log`.
  4. `RevocationLog::open_or_create` prüft `path_buf.exists()`. Da die Datei fehlt, wird die Verzweigung übersprungen. Die Funktion gibt `Ok(Self)` mit **leerem** `revoked_targets`-Set zurück, ohne einen Fehler zu melden!
  5. Wenn ein Angreifer nun `KeyRegistry::decrypt_record` auf den auf Disk verbliebenen `EncryptedRecordPayload` aufruft, prüft `is_group_revoked(42)`. Da das Widerrufslog leer ist, liefert die Prüfung `false`.
  6. `KeyRegistry::get_or_derive` wickelt den in den Datenstrukturen gespeicherten wrapped KEK ab und entschlüsselt die vormalig "geshreddeten" Daten wieder im Klartext.
- **Auswirkung:** Vollständiges Versagen der Durability-Garantie des Crypto-Shreddings. Geshreddete vertrauliche Daten werden nach einem Neustart mit fehlender Log-Datei unbemerkt wieder entschlüsselbar.
- **Empfehlung:** Wenn ein `RevocationLog` mit einem Dateipfad konfiguriert wird, muss der Status der Initialisierung explizit im System-Header/Manifest vermerkt werden. Fehlt die Datei beim Reopen eines bestehenden Clusters, muss das System **fail-closed** mit einem kritischen `IntegrityViolation`-Fehler abbrechen, anstatt stillschweigend einen leeren Zustand zu erzeugen.

---

### [HIGH] F-1 — Unvollständige Signatur-Payload-Bindung bei Version 2 HMAC `DeletionProof` (Timestamp & Metadaten unversiegelt)

- **Ort:** `crates/contextra-crypto/src/deletion_proof.rs:271-298` (`create_with_wal_receipt`), `397-440` (`verify`)
- **Invariante betroffen:** I-1, I-2
- **Beleg:**
```rust
// crates/contextra-crypto/src/deletion_proof.rs:271-285
let signature = compute_hmac_sha256(
    proof_key,
    &[
        &scope_bytes,
        &deleted_keys_hash,
        &tx_bytes,
        &covered_layers_bytes,
        &excluded_scopes_bytes,
        receipt_part,
    ],
)?;
```
- **Angriffs-/Fehlerszenario:**
  1. In Version 2 HMAC-Proofs werden im HMAC-Payload nur `scope`, `deleted_keys_hash`, `deleted_after_tx`, `covered_layers`, `excluded_scopes` und `wal_chain_receipt` signiert.
  2. Die Struct-Felder `timestamp` (Erstellungszeitpunkt), `audit_chain_position` und `integrity_warning` sind **nicht** Bestandteil des HMAC-Payloads.
  3. Ein Angreifer fängt einen gültigen v2-`DeletionProof` ab und verändert das Feld `timestamp` von `0` auf einen beliebigen Unix-Zeitstempel (z. B. den aktuellen Ausführungszeitpunkt).
  4. Der Aufruf `proof.verify(&key)` berechnet den HMAC-SHA256 neu über die 6 signierten Teile. Da `timestamp` nicht im HMAC eingeht, stimmt die berechnete Signatur exakt überein.
  5. `verify()` gibt `Ok(true)` zurück. Ein Auditor akzeptiert einen fälschlich datierten Löschbeweis.
- **Auswirkung:** Manipulation von Zeitstempeln und Audit-Metadaten in v2-Löschbeweisen ohne Verifikationsfehler. Replay- und Rückdatierungs-Angriffe in Compliance-Audits.
- **Empfehlung:** Sämtliche im `DeletionProof`-Struct enthaltenen Felder (insbesondere `timestamp` und `audit_chain_position`) müssen ausnahmslos in den signierten Byte-Payload aufgenommen werden (wie in v3 geschehen).

---

### [HIGH] F-2 — Fehlende Schlüssel-Objekt-Separation bei HMAC v1/v2-Beweisen ermöglicht Beweisfälschung durch Dateisystem-Angreifer

- **Ort:** `crates/contextra-crypto/src/crypto.rs:260-272` (`derive_deletion_proof_key`), `crates/contextra-crypto/src/deletion_proof.rs:271-298`
- **Invariante betroffen:** I-2, I-7
- **Beleg:**
```rust
// crates/contextra-crypto/src/crypto.rs:260-267
pub fn derive_deletion_proof_key(&self) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::from_prk(self.key.as_bytes())
        .map_err(|_| CryptoError::Crypto("Invalid PRK length".to_string()))?;
    let mut key = [0u8; 32];
    hk.expand(b"deletion-proof", &mut key)...;
    Ok(key)
}
```
- **Angriffs-/Fehlerszenario:**
  1. Das System nutzt symmetrische HMAC-SHA256 v1/v2-Beweise. Der Signierschlüssel wird via HKDF direkt vom Hauptschlüssel (`KeyManager`) der Datenbank abgeleitet.
  2. Ein Angreifer erlangt nach einer Datenlöschung Lesezugriff auf das Dateisystem oder ein Backup (z. B. durch einen Datenleck-Vorfall).
  3. Der Angreifer extrahiert die Passphrase/Salt oder den Master-Schlüssel des Clusters.
  4. Der Angreifer berechnet `derive_deletion_proof_key()` extern nach und stellt einen gefälschten v2-HMAC-`DeletionProof` aus, welcher behauptet, dass alle SSTables (`SsTableAllLevels`) und WAL-Segmente gereinigt wurden, obwohl die geleakten SSTables die Quelldaten im Klartext enthalten.
  5. Jeder Prüfer, der denselben Master-Schlüssel zur Verifizierung nutzt, stuft den gefälschten Proof als kryptographisch gültig ein.
- **Auswirkung:** Symmetrische HMAC-Löschbeweise bieten keine Nichtabstreitbarkeit (Non-Repudiation) gegenüber Parteien, die Zugriff auf den Master-Schlüssel besitzen. Angreifer mit Disk-Access können Löschbeweise beliebig fälschen.
- **Empfehlung:** Symmetrische v1/v2 HMAC-Löschbeweise sollten im Produktionsbetrieb als veraltet markiert und schrittweise abgeschaltet werden. Der Erzeugungspfad muss standardmäßig auf v3 Ed25519 mit asymmetrischer Schlüssel-Separation gezwungen werden.

---

### [HIGH] F-4 — `DeletionProofKeyPair` versäumt RAM-Zeroization (`ZeroizeOnDrop`) für Ed25519-Privatschlüssel

- **Ort:** `crates/contextra-crypto/src/deletion_proof.rs:59-83`, `crates/contextra-crypto/src/ed25519_proof.rs:35-64`
- **Invariante betroffen:** I-6
- **Beleg:**
```rust
// crates/contextra-crypto/src/deletion_proof.rs:59-64
#[derive(Debug)]
pub struct DeletionProofKeyPair {
    signing_key: ed25519_dalek::SigningKey,
    pub verifying_key: ed25519_dalek::VerifyingKey,
}
```
- **Angriffs-/Fehlerszenario:**
  1. Ein `DeletionProofKeyPair` wird erzeugt, um v3-Löschbeweise asymmetrisch zu signieren.
  2. Nach der Signierung verlässt die `DeletionProofKeyPair`-Instanz den Gültigkeitsbereich und wird vom Stack/Heap freigegeben.
  3. Da weder `DeletionProofKeyPair` in `deletion_proof.rs` noch in `ed25519_proof.rs` das `#[derive(ZeroizeOnDrop)]`-Attribute trägt, verbleibt der 32-Byte-Geheimschlüssel des Ed25519-Signierschlüssels unbereinigt in nicht-nullierten RAM-Seiten.
  4. Bei einem Core-Dump, Swap-out auf Disk oder Heartbleed-ähnlichen RAM-Lesefehlern kann der private Ed25519-Signierschlüssel extrahiert werden.
- **Auswirkung:** Verletzung der in `crates/contextra-crypto/README.md` dokumentierten Ring-0-Invariante `Safe API (In-Memory Zeroization)`.
- **Empfehlung:** Beide `DeletionProofKeyPair`-Structs müssen mit `#[derive(Zeroize, ZeroizeOnDrop)]` annotiert werden.

---

### [MEDIUM] F-5 — Stille Entwertung von v1/v2 HMAC-Löschbeweisen bei Hauptschlüssel-Rotation (`KeyManager`)

- **Ort:** `crates/contextra-crypto/src/deletion_proof.rs:445-480`, `crates/contextra-crypto/src/crypto.rs:260-272`
- **Invariante betroffen:** I-2, I-7
- **Beleg:**
```rust
// crates/contextra-crypto/src/deletion_proof.rs:445-455
pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool> {
    ...
    match version {
        SignatureVersion::V1 | SignatureVersion::V2 => {
            let expected = compute_hmac_sha256(proof_key, ...)?;
            Ok(expected.as_slice().ct_eq(&self.signature).into())
        }
    }
}
```
- **Angriffs-/Fehlerszenario:**
  1. Das System erzeugt v1/v2 HMAC-Löschbeweise.
  2. Nach 6 Monaten wird der Hauptschlüssel des Clusters routinemäßig rotiert.
  3. Ein Auditor fordert die Verifikation eines vor 5 Monaten ausgestellten Löschbeweises an.
  4. Die Anwendung ruft `proof.verify(&key_manager)` auf. `KeyManager` leitet den HMAC-Schlüssel aus dem **neuen** Hauptschlüssel ab.
  5. Der HMAC-Vergleich schlägt fehl. `verify()` gibt `Ok(false)` zurück.
- **Auswirkung:** Rechtsverbindliche Altbeweise werden nach einer Schlüsselrotation fälschlicherweise als ungültig/gefälscht zurückgewiesen.
- **Empfehlung:** Symmetrische HMAC-Beweise erfordern bei Schlüsselrotation die Führung einer historisierten Schlüssel-Registratur oder die vollständige Migration auf v3 Ed25519.

---

### [MEDIUM] F-6 — Stilles Downgrade-Risiko von v3 Ed25519 auf v1/v2 HMAC-Signaturen in Verifikationspfaden

- **Ort:** `crates/contextra-crypto/src/deletion_proof.rs:388-440` (`verify`), `crates/contextra-crypto/src/ed25519_proof.rs:110-135` (`SignatureVersion`)
- **Invariante betroffen:** I-2
- **Beleg:**
```rust
// crates/contextra-crypto/src/deletion_proof.rs:445-450
pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool> {
    let version = self.signature_version_typed()?;
    let key = key.into();
    ...
```
- **Angriffs-/Fehlerszenario:**
  1. Ein System akzeptiert über `VerificationKey` sowohl HMAC- als auch Ed25519-Schlüssel.
  2. Ein Angreifer nimmt einen v3-Ed25519 Proof, ändert die `signature_version` auf `2` und berechnet eine passende HMAC-Signatur mit einem geleakten HMAC-Schlüssel.
  3. Wenn der Empfänger den Verifikationsaufruf mit `VerificationKey::Hmac` durchführt, wird der manipulierte Proof ohne Versions-Mismatch als v2 HMAC-Proof verifiziert.
- **Auswirkung:** Downgrade-Angriff ermöglicht die Umgehung der strikteren v3-Payload-Prüfung und der Längenpräfix-Hashes.
- **Empfehlung:** Verifikationspfade müssen die zugelassene `SignatureVersion` strikt an den übergebenen `VerificationKey`-Typ binden und HMAC-Verifikation für v3-Payloads unterbinden.

---

## 5. Optimierungspotenzial (Phase D)

1. **Vermeidung transienter Bincode-Allokationen bei v3 Payload-Erstellung (`deletion_proof.rs:502-536`):**
   - *Analyse:* In `DeletionProof::construct_v3_payload()` werden `scope`, `covered_layers`, `excluded_scopes` und `graph_repair` einzeln via `bincode::serialize` in separate `Vec<u8>`-Heap-Puffer serialisiert und anschließend in `payload` kopiert.
   - *Vorschlag:* Nutzung von `bincode::serialize_into` direkt in den vorgehaltenen `payload`-Vektor. Dies spart 4 temporäre Heap-Allokationen pro Signatur- und Verifikationsvorgang.
2. **Lock-freie Revocation-Prüfung in `KeyRegistry` (`kv_shredding.rs:188-202`):**
   - *Analyse:* `KeyRegistry::is_group_revoked` akquiriert bei jedem Lesezugriff ein `RwLock` auf `revoked_groups` und greift auf das `RwLock` des `RevocationLog` zu.
   - *Vorschlag:* Einsatz einer `ArcSwap<HashSet<u64>>`- oder Atomic-Bitset-Kaskade für hochfrequente Gruppen-Lookups, um Lock-Contention im heißen Lese-Pfad zu eliminieren.

---

## 6. Offene Fragen / nicht verifizierbar ohne Laufzeit-Tests

1. **System-Verhalten bei Dateisystem-Vollzustand während `RevocationLog::append`:**
   - In `revocation_log.rs:290-315` wird ein temporäres File erzeugt, synchronisiert (`sync_all`) und per `rename` atomar ausgetauscht. Das Verhalten bei Schreibabbruch mitten im `rename` auf netzwerkbasierten Dateisystemen (NFS/SMB) erfordert Integrationstests auf POSIX-Ebene.

---

## 7. Jules-Task-Karten

```yaml
id: JULES-P04-1
title: RevocationLog-Fehlen fail-closed absichern und Reopen-Prüfung erzwingen
severity: CRITICAL
files_to_touch:
  - crates/contextra-crypto/src/revocation_log.rs
  - crates/contextra-crypto/src/kv_shredding.rs
context: >
  Fehlt die revocation.log-Datei beim Neustart, initialisiert RevocationLog::open_or_create
  stillschweigend ein leeres Widerrufsregister. Dadurch werden vormalig geshreddete
  Schlüssel (KEKs/DEKs) wieder entschlüsselbar (F-3).
acceptance_criteria:
  - Wenn ein Pfad zur revocation.log angegeben ist und die Datei nicht existiert, aber das
    Datenbank-Verzeichnis bereits Schlüssel enthält, muss open_or_create mit CryptoError::IntegrityViolation abbrechen.
  - Das Erzeugen eines neuen leeren Log-Files ist nur bei expliziter Initialisierung gestattet.
test_to_add: >
  Ein Integrationstest test_missing_revocation_log_fails_closed_on_reopen, der ein Group-Revoke
  durchführt, die Log-Datei löscht und beim Re-open einen CryptoError::IntegrityViolation nachweist.
non_goals: >
  Keine Änderung an der Bincode-Serialisierung der RevocationEntry-Struktur.
```

```yaml
id: JULES-P04-2
title: DeletionProof v2 HMAC-Payload um Timestamp und Metadaten erweitern
severity: HIGH
files_to_touch:
  - crates/contextra-crypto/src/deletion_proof.rs
context: >
  In Version 2 HMAC DeletionProofs wird das Feld timestamp nicht in die compute_hmac_sha256-Payload
  einzogen (F-1). Dadurch können Zeitstempel ohne Verifikationsfehler manipuliert werden.
acceptance_criteria:
  - Der Erstellungspfad create_with_wal_receipt muss timestamp.to_le_bytes() in die HMAC-Eingabe aufnehmen.
  - DeletionProof::verify() muss für SignatureVersion::V2 den Zeitstempel im HMAC prüfen.
test_to_add: >
  Ein Unit-Test test_v2_tampered_timestamp_fails_verify, der das timestamp-Feld eines v2-Proofs
  manipuliert und nachweist, dass verify() false zurückgibt.
non_goals: >
  Keine Abkündigung von SignatureVersion::V1.
```

```yaml
id: JULES-P04-3
title: Symmetrische v1/v2 HMAC-Löschnachweise im Erzeugungspfad abkündigen
severity: HIGH
files_to_touch:
  - crates/contextra-crypto/src/crypto.rs
  - crates/contextra-crypto/src/deletion_proof.rs
context: >
  KeyManager::create_deletion_proof leitet den HMAC-Signierschlüssel vom Datenbank-Master-Key ab (F-2).
  Ein Angreifer mit Disk-Access kann Löschnachweise fälschen, da beschütztes Gut und Signierschlüssel
  nicht getrennt sind.
acceptance_criteria:
  - KeyManager::create_deletion_proof muss standardmäßig Ed25519-v3-Löschnachweise erzeugen.
  - Der Aufruf von symmetrischen v1/v2-Erzeugungsmethoden muss als deprecated markiert werden.
test_to_add: >
  Ein Test test_key_manager_creates_v3_asymmetric_proof, der verifiziert, dass create_deletion_proof
  einen v3-Proof mit Ed25519-Signatur ausgibt.
non_goals: >
  Keine Entfernung des v1/v2-Verifikationspfads für bestehende Alt-Beweise.
```

```yaml
id: JULES-P04-4
title: DeletionProofKeyPair mit ZeroizeOnDrop für RAM-Bereinigung ausstatten
severity: HIGH
files_to_touch:
  - crates/contextra-crypto/src/deletion_proof.rs
  - crates/contextra-crypto/src/ed25519_proof.rs
context: >
  DeletionProofKeyPair hält den ed25519_dalek::SigningKey, derives jedoch kein ZeroizeOnDrop (F-4).
  Dies verletzt die Ring-0-Invariante zur In-Memory-Zeroization.
acceptance_criteria:
  - DeletionProofKeyPair in deletion_proof.rs und ed25519_proof.rs deriviert Zeroize und ZeroizeOnDrop.
  - Ed25519-Schlüsselmaterial wird bei Drop im Speicher überschrieben.
test_to_add: >
  Ein Unit-Test test_deletion_proof_keypair_zeroize_on_drop, der die Speicherbereinigung nach Drop verifiziert.
non_goals: >
  Keine Umstellung der ed25519_dalek-Bibliothek.
```

---

## Löschversprechen vs. Realität

| README / Doku-Aussage | Code-Befund | Beleg (Datei & Zeilen) |
|---|---|---|
| *"DSGVO-Art.-17-Löschnachweise (`DeletionProof`) extern verifizierbar"* | **Hält (für v3)** / **Hält nicht (für v1/v2)**: v3 ist asymmetrisch mit Ed25519 extern verifizierbar. v1/v2 erfordern Kenntnis des geheimen HMAC-Schlüssels/Master-Keys. | `crates/contextra-crypto/src/deletion_proof.rs:445-536` |
| *"CryptoShred vernichtet KEK/DEK in O(1) zeitlich unumkehrbar"* | **Hält teilweise**: KEK/DEK wird im RAM sofort gezeroizt. Löschen der `revocation.log`-Datei schaltet das Widerrufsregister nach Neustart jedoch lautlos ab. | `crates/contextra-crypto/src/revocation_log.rs:138-158`<br>`crates/contextra-crypto/src/kv_shredding.rs:188-202` |
| *"In-Memory Zeroization (Safe API, Key Hierarchy)"* | **Hält teilweise**: `VolatileEncryptionKey`, `SubKey` und `KvSegment` zeroizen bei Drop. `DeletionProofKeyPair` versäumt `ZeroizeOnDrop`. | `crates/contextra-crypto/src/deletion_proof.rs:59-64`<br>`crates/contextra-crypto/src/ed25519_proof.rs:35-40` |
| *"DeletionProof garantiert physisches Löschen der Storage-Layer"* | **Hält nicht (by Design laut Report)**: Report deklariert explizit, dass `DeletionProof` nur logisches LSM-Entfernen attestiert. Keine physische SSD-Säuberung. | `docs/reports/deletion_proof_verifiability_report.md:11-20` |
| *"Fälschungssichere Bindung von Lösch-Scope und Zeitstempel"* | **Hält nicht (für v2)**: Der Erstellungszeitstempel `timestamp` ist in v2 HMAC-Proofs nicht im HMAC-Payload gebunden und beliebig manipulierbar. | `crates/contextra-crypto/src/deletion_proof.rs:271-285` |
