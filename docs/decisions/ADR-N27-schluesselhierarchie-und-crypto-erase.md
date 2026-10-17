# ADR-N27: Schlüsselhierarchie und Crypto-Erase für Dokumentdaten (Verschlüsselung-at-Rest im Profil Sovereign)

* **Status:** Proposed
* **Datum:** 2026-10-17
* **Autoren:** Principal Architect und Kryptografie-Reviewer (Contextra)
* **Betroffene Komponenten:** `crates/contextra-crypto`, `crates/contextra-store`, `crates/contextra-engine`, `crates/contextra-types`

---

## 1. Ist-Analyse der bestehenden Kryptografie-Architektur

Eine systematische Überprüfung der bestehenden Codebasis zeigt, dass Verschlüsselung und Schlüsselverwaltung in Contextra derzeit auf spezifische Teilbereiche beschränkt sind und keine durchgängige Schlüsselhierarchie für Dokumentdaten in der Storage Engine existiert:

1. **Master-Schlüsselverwaltung (`KeyManager`):**
   * **Implementierung:** In `crates/contextra-crypto/src/crypto.rs` verwaltet der `KeyManager` den Hauptschlüssel (Master PRK) [V]. Die Schlüsselableitung erfolgt über Argon2id (`try_new_argon2id` / `try_new_with_kdf`) gemäß `INV-KDF-1` [V].
   * **Nonce-Design:** Das Modul nutzt AES-256-GCM-SIV (RFC 8452) für Nonce-Misuse-Resistance [V]. Die Nonce-Generierung erfolgt in `encrypt_auto_nonce` über ein 4-Byte random `nonce_prefix` (über `OsRng`) kombiniert mit einem atomaren 8-Byte `AtomicU64`-Zähler [V].
   * **Subkey-Ableitung:** `KeyManager` bietet Subkey-Ableitungen via HKDF-SHA256 für Dateien (`derive_file_key`), KV-Segmente (`derive_kv_key`, `derive_segment_key`) und HMAC-Integritätsketten (`integrity_key`) [V].

2. **In-Memory KV-Cache & KV-Shredding (`KvSegmentCipher` & `KeyRegistry`):**
   * **`KvSegmentCipher` (`crates/contextra-crypto/src/kv_cipher.rs`):** Verschlüsselt In-Memory-KV-Cache-Aktivierungstensoren und bindet Schlüssel strikt an das Tupel `(TenantId, ModelFingerprint)` via HKDF-Domain-String `"contextra-kv-layer-v2:"` [V].
   * **`KeyRegistry` & `kv_shredding` (`crates/contextra-crypto/src/kv_shredding.rs`):** Implementiert Envelope Encryption für KV-Cache-Segmente mit Gruppen-Schlüsseln (`GroupKek`) und Record-Schlüsseln (`RecordDek`) [V]. Bietet $O(1)$ Schlüsselwiderruf (`revoke_group`, `revoke_record`, `shred_segment_key`) [V]. Der Widerrufsstatus wird im persistenten `RevocationLog` protokolliert [V].

3. **WAL- und Storage-Engine-Verschlüsselung (`contextra-store`):**
   * **WAL-Kryptografie (`crates/contextra-crypto/src/wal_crypto.rs`, `crates/contextra-store/src/wal/io.rs`, `flusher.rs`):** Bietet HMAC-SHA256-Integritätsverifikation (`WalHmac`, `IntegrityVerifier`) sowie optionale Frame-Verschlüsselung (Magic Header `MFW2`) [V].
   * **SSTable-I/O (`crates/contextra-store/src/sstable/io.rs`, `crates/contextra-store/src/lsm/engine.rs`):** Liest und schreibt SSTable-Blöcke. Ist `encryption_passphrase` gesetzt, werden Blöcke über den aus der Passphrase abgeleiteten `KeyManager` bzw. `derive_file_key` verschlüsselt [V].

4. **LSM Engine Default-Konfiguration (`crates/contextra-store/src/lsm/config.rs`):**
   * **Standardwert:** `LsmConfig::default()` setzt `encryption_passphrase: None` [V]. Verschlüsselung ruhender Daten (Encryption-at-Rest) ist somit standardmäßig **deaktiviert** [V].
   * **Fehlende Dokument-Schlüsselhierarchie:** Es existiert derzeit **keine Schlüsselhierarchie für Dokumentdaten** in SSTables und WAL. Alle Datenblöcke einer Datenbankinstanz werden bei aktivierter Verschlüsselung mit demselben Master-Key bzw. datei-abgeleiteten Schlüssel verschlüsselt [V]. `kv_shredding` und `RevocationLog` decken ausschließlich das KV-Segment ab, nicht jedoch Dokumente oder Vektoren im LSM-Tree [V].

---

## 2. Optionen-Analyse zur Erweiterung der Schlüsselhierarchie

Um Crypto-Erase (Schlüsselvernichtung gemäß DSGVO Art. 17 und NIST SP 800-88 Rev. 2) auf Dokumentenebene oder Mandantenebene bereitzustellen, werden drei Architektur-Optionen evaluiert:

### Option A: DEK pro Collection-Inkarnation + KEK pro Tenant-Epoche (Empfohlen)

* **Konzept:** Dreistufige Schlüsselhierarchie. Der Master-Key (aus Passphrase/KMS) verschlüsselt den Tenant-KEK (Key Encryption Key). Der Tenant-KEK verschlüsselt den Collection-DEK (Data Encryption Key) für eine spezifische Collection-Inkarnation.
* **Löschgranularität:** Collection-Ebene (vollständiges Crypto-Erase aller Dokumente einer Collection durch Vernichtung des Collection-DEKs) sowie Tenant-Ebene (Tenant-Reset durch Vernichtung des Tenant-KEKs). Einzelne Dokument-Löschungen innerhalb einer aktiven Collection erfolgen weiterhin über physische Purge-Compactions gemäß ADR-N21 [S].
* **Format-Änderungen:** SSTable-Block-Header und WAL-Frame-Header werden um `tenant_epoch_id` (u64 LE) und `collection_incarnation_id` (u64 LE) erweitert.
* **Migration:** Unverschlüsselte oder V1-Master-Key-verschlüsselte SSTables/WALs werden im Hintergrund-Compaction-Schritt (`compaction/engine.rs`) transparent in das V2-Collection-DEK-Format umgeschrieben [V].
* **Leistung:** Vernachlässigbarer Overhead. KEK- und DEK-Schlüssel werden im RAM-Cache der `KeyRegistry` gehalten ($O(1)$ RwLock-Lookup) [V]. Keine zusätzlichen I/O-Zugriffe pro Dokument oder Scan.
* **Schlüsselverwahrung:** KEKs und DEKs werden als `wrapped_kek` / `wrapped_dek` mit AES-256-GCM-SIV verschlüsselt im `KeyRegistry`-Store/`RevocationLog` abgelegt [V]. Master-Key wird aus einer Datei mit Zugriffsrechten `0600`, einer Argon2id-Passphrase oder einem externen KMS (mit einstellbarer Quarantäne-Wartefrist, z. B. 72h gemäß `DEFAULT_COLLECTION_QUARANTINE_NANOS` in `kv_shredding.rs`) geladen [V].
* **Backups:** Ein Backup des LSM-Storage ohne den (separat im KMS aufzubewahrenden oder nach Löschung vernichteten) Collection-DEK/Tenant-KEK ist kryptografisch unlesbar und absolut unumkehrbar.

### Option B: DEK pro Dokument mit zentraler Schlüsseltabelle

* **Konzept:** Jedes einzelne Dokument wird mit einem eigenen, zufälligen Dokument-DEK verschlüsselt. Die DEKs werden in einer zentralen, verschlüsselten Indirektionstabelle (Schlüsselspeicher) verwaltet.
* **Löschgranularität:** Einzelnes Dokument ($O(1)$ Crypto-Erase pro Dokument ohne Compaction).
* **Format-Änderungen:** Jeder SSTable-Dokument-Eintrag und jeder WAL-Frame benötigt ein `doc_dek_id` (128-Bit UUID) sowie ein `wrapped_dek`-Feld im Header.
* **Migration:** Erfordert ein vollständiges Re-Wrapping aller bestehenden Dokumente und den Aufbau einer gigantischen Indirektionstabelle.
* **Leistung:** **Erheblicher Performance-Einbruch.** Jeder Point-Lookup und jeder Range-Scan erfordert einen vorgelagerten Schlüssel-Lookup in der Schlüsseltabelle. Dies verletzt die Random-Read- (K2) und Range-Scan-Throughput-Vorgaben (K3) aus ADR-N20 massiv [S].
* **Schlüsselverwahrung:** Die Schlüsseltabelle wächst linear mit der Anzahl der Dokumente (Millionen von DEKs im RAM/Disk), was zu extremer Memory- und Storage-Amplifikation führt.
* **Backups:** Sehr hoher Verwaltungsaufwand; Vernichtung eines Dokument-DEKs macht das Dokument in allen historischen Backups unlesbar, erfordert jedoch synchrone Schlüsseltabellen-Sicherungen.

### Option C: Nur physischer Purge (Kein Crypto-Erase für Dokumentdaten / Kein S4)

* **Konzept:** Beibehaltung des Status Quo für Dokumentdaten. Löschungen erfolgen ausschließlich über physische Compaction und File Scrubbing (`scrub_if_safe` / `PendingDeleteJournal` in `crates/contextra-durable-fs/src/scrub.rs`) [V].
* **Löschgranularität:** Nur physische Sektor-Bereinigung. Kein $O(1)$ Crypto-Erase.
* **Format-Änderungen / Migration / Leistung:** Keine Formatänderungen, kein Krypto-Overhead.
* **Schlüsselverwahrung:** Einfacher Master-Key / Passphrase.
* **Backups:** **Sicherheitsdefizit:** Auf immutable Cloud-Speichern (S3, Copy-on-Write Blockstorage) oder alten Backup-Snapshots verbleiben gelöschte Dokumente dauerhaft unverschlüsselt bzw. lesbar [S]. Kein S4-Löschnachweis (`WitAssurance::S4`) für Dokumentdaten möglich [V].

---

## 3. Empfehlung, Sovereign Default & Reversibilitäts-Bedingung

### Empfehlung

Es wird **Option A (DEK pro Collection-Inkarnation + KEK pro Tenant-Epoche)** als strategische Architektur für Contextra beschlossen.

* **Begründung:** Option A stellt das optimale Gleichgewicht zwischen mathematisch nachweisbarem Crypto-Erase (S4 Assurance für Collections und Tenants) und maximaler I/O-Performanz dar. Sie fügt sich nahtlos in ADR-N21 (Physische Bereinigung auf Dokumentebene, Crypto-Shredding auf Scope/Collection/Tenant-Ebene) und ADR-102 (Tenant-Scoped Storage) ein [S].

### Verschlüsselungs-Default für das Profil `sovereign`

1. Für das Profil `sovereign` (`FeatureRing::Sovereign`) wird **Option A mit aktivierter Verschlüsselung (`encryption-at-rest`) als verflichtender Standard** festgelegt [S].
2. Der Start einer Datenbankinstanz im Profil `sovereign` ohne konfigurierte Passphrase oder KMS-Schlüsselanbindung wird strikt fail-closed mit `DurabilityConfigError::IncompatibleCombination` abgelehnt [V].

### Reversibilitäts-Bedingung

Eine Abweichung von Option A hin zu Option B (Dokument-DEKs) wird nur dann in Erwägung gezogen, wenn durch hardwareseitige Self-Encrypting Drives (SED) mit Per-Page-Hardware-Key-Management der Software-Schlüssel-Lookup-Overhead bei $1.000.000$ Dokumenten nachweislich unter $5\%$ Range-Scan-Latenzaufschlag bleibt [S].

---

## 4. Nachweis, Zeugen & Rechtlicher Pflicht-Vorbehalt

### Zeuge „Keyring" im Löschbeweis

* **Evidenz-Struktur:** Der Löschnachweis für den Schlüsselring ist in `crates/contextra-crypto/src/deletion_proof/evidence.rs` über die Variante `WitLayerEvidence::Keyring` (Variant Tag `0x0001 = 10`, Layer ID 25 in `deletion_proof/layer_ids.rs`) definiert [V].
* **Invarianten:** Gemäß `check_invariants()` erfordert die Gültigkeit des Zeugnisses zwingend `dek_present == false` (DEK aus dem Speicher gelöscht) und `kek_epoch_destroyed == true` (KEK-Epoche dauerhaft vernichtet) [V].
* **Assurance-Level:** Der Zeuge „Keyring" erbringt den Nachweis für **S4 Cryptographic Erasure Assurance** (`WitAssurance::S4`) gemäß `deletion_witness.rs` [V].

### Pflicht-Vorbehalt zum offenen Rechtsstatus (Formeller Disclaimer)

> **Rechtlicher Pflicht-Vorbehalt:** NIST SP 800-88 Rev. 2 (26.09.2025) erkennt Crypto-Erase (Schlüsselvernichtung) als valide Purge-Methode für elektronische Speichermedien an, sofern die kryptografischen Schlüssel mit mindestens 128 Bit Entropie vernichtet werden und der Chiffretext ohne Kenntnis des Schlüssels mathematisch unlesbar ist [S]. Der datenschutzrechtliche Status von Crypto-Erase unter DSGVO Art. 17 ist jedoch europarechtlich offen: Weder der Europäische Datenschutzausschuss (EDPB) noch der Europäische Gerichtshof (EuGH) haben bisher ausdrücklich bestätigt, dass die bloße Schlüsselvernichtung ohne physische Sektorüberschreibung die rechtlichen Anforderungen an eine vollständige Löschung personenbezogener Daten erfüllt [S]. Sämtliche Aussagen in diesem Dokument stellen keine Rechtsberatung dar und begründen keine rechtliche Haftung [S].

### Kryptografische Grundannahmen

1. **Entropie & CSPRNG:** Sämtliche KEKs, DEKs und Nonce-Prefixes werden über den System-CSPRNG (`rand::rngs::OsRng`) mit 256 Bit Entropie erzeugt [V].
2. **Schlüssellänge & Algorithmus:** Verwendung von AES-256-GCM-SIV (256-Bit Schlüssel, 12-Byte Nonce) für alle Datenblöcke [V].
3. **Schlüsselverwahrung & Zeroization:** Sämtliche unverschlüsselten Schlüssel im Arbeitsspeicher sind in `ZeroizeOnDrop`-Typen (`SubKey`, `GroupKek`, `RecordDek` in `crates/contextra-crypto/src/kv_shredding.rs`) gekapselt [V]. Dateibasierte Schlüssel-Stores erzwingen POSIX-Zugriffsrechte `0600` [V].

---

## 5. Migrationsplan & Teststrategie

### Migrationsplan für Bestands-Datenbanken

1. **Phase 1 (Reopen & Header Inspection):**
   Beim Öffnen einer bestehenden Datenbank prüft `LsmStorage::open` den Format-Header. Unverschlüsselte oder V1-Master-Key-SSTables erhalten im In-Memory-Manifest ein `legacy_format`-Flag [V].
2. **Phase 2 (Lazy Compaction Rewrite):**
   Sobald Hintergrund-Compactions (`compaction/engine.rs`) getriggert werden, liest der `SstableStream` Alt-Blöcke und schreibt neue SSTables unter Verwendung der aktiven Collection-DEKs aus dem `KeyRegistry` [V].
3. **Rückfallpfad (Downgrade/Rollback):**
   Solange die KEK/DEK-Einträge im `KeyRegistry` / `RevocationLog` aktiv und nicht vernichtet sind, können SSTables über ein Migrationstool (`xtask migrate-key-hierarchy --downgrade`) in das legacy Master-Key-Format zurückentschlüsselt werden [V]. Sobald der Schlüssel vernichtet wurde (`revoke_group`), ist kein Rollback mehr möglich (fail-closed) [V].

### Teststrategie

1. **Testfall 1 (Schlüsselvernichtung → Entschlüsselung scheitert):**
   * **Ablauf:** Erzeugen einer Collection, Schreiben von Dokumenten in SSTable/WAL mit Collection-DEK. Anschließend Aufruf von `KeyRegistry::revoke_group(collection_incarnation_id)`.
   * **Erwartung:** Jeder nachfolgende Lese- oder Replay-Versuch muss sofort mit `CryptoError::KeyRevoked` / `CryptoError::Crypto` fehlschlagen. Es dürfen keine Klartextdaten rekonstruierbar sein [V].
2. **Testfall 2 (Restore alter Backups bleibt unlesbar):**
   * **Ablauf:** Erstellen eines Datei-Backups der verschlüsselten SSTable/WAL-Dateien. Schlüsselvernichtung im primären `RevocationLog` / KMS durchführen. Wiederherstellung des Backups in einer neuen Datenbankinstanz versuchen.
   * **Erwartung:** Die wiederhergestellte Instanz kann die Blöcke ohne den vernichteten Schlüssel nicht öffnen. Der Scan bricht mit `CryptoError::KeyRevoked` ab [V].

---

## 6. Offene Fragen an den Product Owner

1. **Quarantäne-Dauer für Collection-DEKs:** Sollen Collection-DEKs beim Aufruf von `drop_collection` sofort unwiderruflich gelöscht werden ($0\text{h}$ Quarantäne) oder soll die definierte Standard-Quarantäne von $72\text{h}$ (`DEFAULT_COLLECTION_QUARANTINE_NANOS` in `kv_shredding.rs`) angewendet werden, um versehentliche administrative Löschungen abzufedern?
2. **Externes KMS-Schnittstellen-Protokoll:** Reicht für erste Unternehmens-Deployments im Profil `sovereign` die lokale dateibasierte Schlüsselverwaltung mit Dateirechten `0600` und Argon2id-Passphrase aus, oder ist die Anbindung an ein KMIP- bzw. PKCS#11-kompatibles externes KMS (z. B. HashiCorp Vault, AWS KMS) ein zwingendes Release-Kriterium für Version 1.0?
3. **Umgang mit Alt-Backups vor der Schlüsselvernichtung:** Reicht der rechtliche Pflicht-Vorbehalt zur Dokumentation gegenüber Unternehmenskunden aus, oder muss in der Produkt-UI/CLI eine explizite Warnung ausgegeben werden, wenn ein Backup ohne externe KMS-Schlüsselanbindung erstellt wird?
