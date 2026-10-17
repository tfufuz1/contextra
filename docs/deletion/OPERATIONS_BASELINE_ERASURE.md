# Betriebsvorgabe zur Löschung (OPERATIONS_BASELINE_ERASURE.md)

**Hinweis:** Vorlage, keine Rechtsberatung.

---

### Zweck & Anwendungsbereich

Dieses Dokument definiert die verbindlichen betrieblichen Grundanforderungen (*Operations Baseline*) für den sicheren und DSGVO-konformen Betrieb einer Contextra-Node.
Contextra ist als **Single-Node-Anwendung** konzipiert. Da die Garantien der Anwendung auf der logischen und kryptographischen Ebene enden, muss die unterlagerte Betriebsumgebung (Betriebssystem, Dateisystem, Hypervisor, Storage, Cloud-Infrastruktur) gehärtet werden, um Restrisiken der physikalischen Speicherung zu minimieren.

---

### Implementierungs- & Beobachtbarkeitsstatus (Operations Baseline)

| Bereich / Baustein | Status | Beobachtbarkeit durch Contextra | Anmerkung / Code-Anker |
| :--- | :---: | :---: | :--- |
| **Full-Disk Encryption (FDE)** | Implementiert | **Beobachtet** | Inspektion via `/proc/self/mountinfo` (`crates/contextra-sys/src/env_observe.rs:418`) |
| **Swap-Speicher** | Implementiert | **Beobachtet** | Inspektion via `/proc/swaps` (`crates/contextra-sys/src/env_observe.rs:427`) |
| **Core Dumps & Crash Dumps** | Implementiert | **Beobachtet / Gehärtet** | Setzt `RLIMIT_CORE = 0` (`crates/contextra-sys/src/hardening.rs:36`) |
| **Dateisystem-Erkennung (CoW vs. Standard)** | Implementiert | **Beobachtet** | Erkennt CoW/Btrfs via Mountinfo (`crates/contextra-sys/src/env_observe.rs:418`) |
| **Btrfs/ZFS-IOCTL Direct Scrubbing** | Geplant | **—** | Nicht in Produktion implementiert; POSIX-Scrubbing-Fallback in `contextra-durable-fs` |
| **S3 Object-Lock & Governance Verification** | Geplant | **—** | Nicht in Produktion implementiert |
| **Cloud-KMS Wartefrist-Tracking (AWS/GCP)** | Geplant / Teilweise | **Protokolliert** | Ausstehende Key-Deaktivierungen manuell / extern verwaltet |

---

### Verbindliche Betriebsvorgaben & Beobachtbarkeit durch Contextra

Jede Anforderung führt explizit auf, ob Contextra ihren Status auf Node-Ebene **beobachtet** (z. B. durch Pre-Flight-Checks, OS-Mountinfo-Inspektion, File-Attribut-Scans) oder **nicht prüfen kann** (da sie rein betrieblich/infrastrukturell außerhalb der Applikationsgrenze liegt).

| Nr. | Betriebsbereich / Anforderung | Technische Vorgabe & Risikobewertung | Status / Beobachtbarkeit durch Contextra |
| :---: | :--- | :--- | :--- |
| **1** | **Full-Disk Encryption (FDE)** | Das gesamte Blockdevice für Betriebssystem und Contextra-Datenverzeichnis muss mit LUKS2 / dm-crypt (unter Linux) oder BitLocker (unter Windows) verschlüsselt sein. | **Beobachtet:** Contextra prüft bei Start über `/proc/self/mountinfo` bzw. OS-Schnittstellen (`crates/contextra-sys/src/env_observe.rs:418`), ob das Datenverzeichnis auf einem verschlüsselten Blockdevice liegt. Ist dies nicht verifizierbar, wird eine Systemwarnung emittiert. |
| **2** | **Swap-Speicher** | OS-Swap muss entweder vollständig deaktiviert (`swapoff -a`) oder mittels eines flüchtigen, bei jedem Boot neu generierten Schlüssels verschlüsselt sein (`dm-crypt` mit `/dev/urandom`). | **Beobachtet:** Pre-Flight-Checks lesen `/proc/swaps` / OS-Metrics (`crates/contextra-sys/src/env_observe.rs:427`). Aktivierter, unverschlüsselter Swap führt zu einer Warnung / Policy-Verletzung. |
| **3** | **Core Dumps & Crash Dumps** | Anwendungs- und Kernel-Core-Dumps müssen systemweit deaktiviert sein (`ulimit -c 0`, `coredump.conf: Storage=none`), um das Auslesen von Schlüsselmaterial (`Zeroize`) aus Process-Memory-Dumps zu verhindern. | **Beobachtet / Gehärtet:** Contextra setzt aktiv `RLIMIT_CORE` auf `0` und prüft Ressource-Limits beim Start (`crates/contextra-sys/src/hardening.rs:36`). |
| **4** | **Hibernation / Sleep States** | Ruhezustand (*Suspend-to-Disk / Hibernation*) muss deaktiviert sein, damit RAM-Inhalte mit KEK/DEK-Schlüsselmaterial nicht unverschlüsselt auf die Disk geschrieben werden. | **Nicht prüfbar:** Liegt in der Verantwortung der OS-Betriebsführung / Systemd-Konfiguration. |
| **5** | **Hypervisor-Snapshots & Live-Migration** | VM-Snapshots, RAM-Captures und Live-Migrationen bergen das Restrisiko, flüchtige Schlüsselzustände einzufrieren oder auf Sekundärspeicher zu kopieren. | **Nicht prüfbar:** Aus der Gast-VM heraus nicht zuverlässig feststellbar; Infrastruktur-Restrisiko. |
| **6** | **CoW-Dateisysteme & Snapshots** | Bei Copy-on-Write-Dateisystemen (z. B. Btrfs, ZFS) führen In-Place-Scrubbing-Versuche zu neuen Chunks statt Überschreibung. Snapshots verhindern die Zerstörung des physischen Blocks. | **Beobachtet:** `contextra-sys` liest `/proc/self/mountinfo` aus (`crates/contextra-sys/src/env_observe.rs:418`). Direct Btrfs/ZFS-IOCTLs sind ein *geplantes Feature*; aktuell erzwingt Contextra bei erkannten CoW-Dateisystemen Crypto-Erase als primären Pfad. |
| **7** | **SED-Warnung (Self-Encrypting Deception)** | Hardware-SEDs (Self-Encrypting Drives) versprechen "Instant Sanitize", verschleiern jedoch oft Firmware-Bugs, fehlende Entropie oder unvollständige Schlüssellöschung. | **Nicht prüfbar:** Contextra verlässt sich **niemals** auf SED-Hardwarelöschung allein, sondern erzwingt stets FDE auf Betriebssystemebene und softwareseitiges Crypto-Key-Shredding (`crates/contextra-crypto/src/revocation_log.rs:285`). |
| **8** | **S3 Delete Marker & Object-Lock** | Bei Nutzung von S3/S3-kompatiblem Objekt-Speicher führen einfache Delete-Requests nur zu *Delete Markers*; versionierte Objekte und *Object-Lock (Governance/Compliance)* verhindern die Löschung. | **Geplant:** Automatische S3-Object-Lock Header-Prüfungen sind als zukünftiges Modul geplant; aktuell nicht in Produktion integriert. |
| **9** | **KMS-Wartefristen (AWS / Google Cloud KMS)** | Externe Cloud-KMS-Dienste erzwingen bei Schlüssellöschung Sicherheits-Wartefristen (AWS KMS: 7–30 Tage; Google Cloud KMS: 24 Stunden bis 120 Tage; typischer Branchenstandard: 45 Tage bis zur endgültigen physikalischen Entfernung). | **Protokolliert / Geplant:** Vermerken ausstehender KMS-Schlüssel-Deaktivierungen im `ErasureLedger` mit Status `PendingDestruction` ist als erweitertes Feature vorgesehen. |

---

### Empfohlene Betriebskonfiguration (Single-Node Reference Setup)

```ini
# Systemd / OS Environment Hardening Checklist
[Service]
LimitCORE=0
MemoryDenyWriteExecute=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/contextra
```

---
