# ADR-N21: F6 Löschstrategie — Physische Bereinigung im Standard-Profil, Crypto-Shredding im Profil sovereign

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Anforderungen an die Löschung von Daten variieren je nach regulatorischem Umfeld und Betriebsumgebung beträchtlich.
  Während in lokalen Standard-Installationen eine physische Überschreibung und SSTable-Kompaktsierung (Physische Bereinigung / Purge) zur DSGVO-Konformität ausreicht, bieten unkontrollierbare Cloud-Speichermedien (z. B. Copy-on-Write-Blockstorage, S3-Objektspeicher oder Flash Translation Layers in SSDs) keine Garantie für das physische Löschen überschriebener Disk-Sektoren.
  Hier erfordert das Bereitstellungsprofil `sovereign` eine mathematische Löschgarantie, bei der Daten unumkehrbar unlesbar gemacht werden, selbst wenn physische Sektor-Fragmente auf den Speichermedien verbleiben (Gesamtspezifikation §F6).

## Entscheidung

1. **Zweistufiges Löschmodell je Profil:**
   * **Standard-Profil (`KvDeleteMode::TombstoneOnly` / Standard):**
     Verwendet physische Bereinigung (`PurgeOrchestrator`). Löschungen schreiben LSM-Tombstones, führen gezielte Purge-Compactions durch und überschreiben/unlinken physische SSTable-, Blob- und WAL-Dateien [V]. Das Dateisystem wird über den `PendingDeleteJournal` bereinigt und gesynct (`scrub_if_safe`) [V].
   * **Profil `sovereign` (`KvDeleteMode::CryptoShred` / `encryption-at-rest`):**
     Verwendet kryptografisches Vernichten (Crypto-Shredding). Sämtliche KV-Segment- und Scope-Daten werden mit eigenen kryptografischen Schlüsseln verschlüsselt (`CryptoKey` / `KvShreddingWiring`) [V]. Beim Löschen wird der spezifische Schlüssel im Schlüsselspeicher und im `Zeroizing`-Speicher physisch zerstört (`shred()`) [V]. Ohne den Schlüssel wird der auf immutable Cloud-Medien verbleibende Chiffretext unumkehrbar zu mathematischem Rauschen.

2. **Verbindliche Invariante (INV-KV-DELETE-1):**
   Gleichzeitiger Betrieb von KV-Löschbeweisen ohne aktivierte Verschlüsselung ist unzulässig ("Ohne Verschlüsselung ist kein KV-Löschbeweis möglich"). `CryptoShred` erfordert zwingend das Feature `encryption-at-rest` [V].

## Alternativen

### Option A: Universelles Crypto-Shredding für alle Installationen
* **Vorteile:** Eine einzige Löschlogik für alle Umgebungen.
* **Nachteile:** Zusätzlicher Verschlüsselungs-Overhead auch dort, wo lokale physische Dateisystem-Bereinigung völlig ausreichend ist.
* **Bewertung:** Verworfen [S].

### Option B: Reine Tombstone-Löschung ohne Crypto-Shredding
* **Vorteile:** Keine Schlüsselverwaltung notwendig.
* **Nachteile:** Unzureichend für Cloud-CoW-Speicher und Hochsicherheits-Umgebungen im `sovereign`-Profil.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Flexible Compliance:** Anpassung an lokale Bare-Metal- und Cloud-Sovereign-Szenarien [V].
* **Garantierte Unlesbarkeit:** Crypto-Shredding garantiert die Unumkehrbarkeit der Löschung unabhängig von Festplatten-Firmware oder Cloud-Snapshots [V].
* **Keine Silent Downgrades:** Das System prüft beim Start die Konsistenz von Löschmodus und Verschlüsselungseinstellungen [V].

## Reversibilitäts-Bedingung

Eine Änderung erfordert neue Hardware-Standardisierungen (z. B. universelle hardware-erzwingende Self-Encrypting Drives mit automatisierter Secure Erase API), die softwareseitiges Crypto-Shredding obsolet machen [S].
