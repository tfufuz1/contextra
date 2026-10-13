# Contextra — Plattformumfang & Schlüsselverwahrung (PLATFORM_SCOPE.md)

**Stand:** 2026-10-12
**Autor:** Governance Architecture Board / Systemarchitekt
**Status:** Normativ

---

## 1. Zielplattformen & Dateisystem-Garantien

Contextra ist als eingebettete High-Performance Storage Engine auf maximale Zuverlässigkeit bei I/O- und `fsync`-Operationen ausgelegt. Die vertraglichen Garantien bezüglich Crash-Sicherheit und Dauerhaftigkeit variieren je nach Ziel-Betriebssystem und Dateisystem.

### 1.1 Garantierte Plattformen (Garantierte Dauerhaftigkeit)

* **Betriebssysteme:** Linux (Kernel ≥ 5.10)
* **Architekturen:** `x86_64`, `aarch64`
* **Dateisysteme:** `ext4` (mit `data=ordered` oder `data=journal`), `xfs`

*Auf den garantierten Plattformen sichert Contextra synchrone Spülungen (`fsync` / `fdatasync`) und atomare Ersetzungen (`renameat2` mit `RENAME_EXCHANGE` / `RENAME_NOREPLACE`) vollständig ab.*

### 1.2 Best-Effort Plattformen (Ohne vertragliche Dauerhaftigkeitsgarantie)

Für Entwicklung, lokale Tests und andere Betriebssysteme gilt ausschließlich **Best-Effort-Support**. Es werden keine harten Garantien gegen Datenverlust bei plötzlichem Stromausfall übernommen:

* **macOS (Darwin):** Best Effort. Verwendet `fcntl(fd, F_FULLFSYNC)` zur Entleerung der Laufwerks-Caches, unterliegt jedoch Einschränkungen bei APFS-Copy-on-Write-Kaskaden.
* **Windows (Win32 / NTFS):** Best Effort. Verwendet `FlushFileBuffers` und Win32-ACLs.
* **Netzwerk-Dateisysteme (NFS, SMB/CIFS, SSHFS):** Best Effort, ausdrücklich **ohne Garantie**. Netzwerk-Caches und POSIX-Locking-Inkompatibilitäten können Atomaritätsgarantien verletzen.
* **Overlay- & Container-Dateisysteme (OverlayFS, Docker-Layers):** Best Effort, ausdrücklich **ohne Garantie**. Fsync auf Copy-Up-Schichten bietet keine physikalische Sektor-Garantie.

---

## 2. Schlüsselverwahrungs-Stufen (Key Management Tiers)

Für die kryptographische Löschung (Crypto-Shredding via `RevocationLog` / `KeyRegistry`) definiert Contextra drei Schlüsselverwahrungs-Stufen:

### K0: Lokale Datei (Best Effort)
* **Beschreibung:** Master-Schlüssel und Revocation-Logs werden als verschlüsselte lokale Dateien auf dem Speichermedium abgelegt (`revocation.log`).
* **Sicherheitsniveau:** Best Effort (lokaler Dateischutz, Win32 ACLs / POSIX 0600).
* **Verwendung:** Lokale Entwicklung, Einzelagenten-Deployments.

### K1: OS-Keychain / TPM / Secure Enclave
* **Beschreibung:** Der Master-Schlüssel zur Entschlüsselung des Revocation-Logs wird im Schlüsselspeicher des Betriebssystems oder Hardware-Sicherheitsmoduls verwahrt (Linux `keyctl` / TPM2, macOS Keychain / Secure Enclave, Windows DPAPI / Credential Manager).
* **Sicherheitsniveau:** Hoch (Hardware-bound oder OS-protected).
* **Verwendung:** Production Edge Devices, Desktop-Anwendungen.

### K2: Externer KMS / HSM (Spezifikation für künftige Arbeiten)
* **Beschreibung:** Die Schlüsselverwaltung und Revocation-Validierung wird an einen externen Cloud-KMS (z. B. AWS KMS, HashiCorp Vault, Azure Key Vault) oder Hardware-Sicherheitsmodul (HSM via PKCS#11) ausgelagert.
* **Sicherheitsniveau:** Sehr hoch (Zentrales Audit, Un-bypassable Revocation).
* **Verwendung:** Enterprise Shared & Cloud Deployments.

---

## 3. Anforderung an Löschnachweise (Deletion Proofs)

**Verbindliche Vorgabe:**
Jedes generierte kryptographische Löschzertifikat (`DeletionProof` v3/v4) muss die verwendete Schlüsselverwahrungs-Stufe (**K0**, **K1** oder **K2**) explizit im Proof-Payload attestieren. Ein externer Verifizierer kann dadurch unmittelbar feststellen, auf welchem Sicherheitsniveau die Schlüssel-Entsorgung vollzogen wurde.
