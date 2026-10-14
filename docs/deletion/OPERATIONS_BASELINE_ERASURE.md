# Betriebsvorgabe zur Löschung (OPERATIONS_BASELINE_ERASURE.md)

**Hinweis:** Vorlage, keine Rechtsberatung.

---

### Zweck & Anwendungsbereich

Dieses Dokument definiert die verbindlichen betrieblichen Grundanforderungen (*Operations Baseline*) für den sicheren und DSGVO-konformen Betrieb einer Contextra-Node.
Contextra ist als **Single-Node-Anwendung** konzipiert. Da die Garantien der Anwendung auf der logischen und kryptographischen Ebene enden, muss die unterlagerte Betriebsumgebung (Betriebssystem, Dateisystem, Hypervisor, Storage, Cloud-Infrastruktur) gehärtet werden, um Restrisiken der physikalischen Speicherung zu minimieren.

---

### Verbindliche Betriebsvorgaben & Beobachtbarkeit durch Contextra

Jede Anforderung führt explizit auf, ob Contextra ihren Status auf Node-Ebene **beobachtet** (z. B. durch Pre-Flight-Checks, OS-Mountinfo-Inspektion, File-Attribut-Scans) oder **nicht prüfen kann** (da sie rein betrieblich/infrastrukturell außerhalb der Applikationsgrenze liegt).

| Nr. | Betriebsbereich / Anforderung | Technische Vorgabe & Risikobewertung | Status / Beobachtbarkeit durch Contextra |
| :---: | :--- | :--- | :--- |
| **1** | **Full-Disk Encryption (FDE)** | Das gesamte Blockdevice für Betriebssystem und Contextra-Datenverzeichnis muss mit LUKS2 / dm-crypt (unter Linux) oder BitLocker (unter Windows) verschlüsselt sein. | **Beobachtet:** Contextra prüft bei Start über `/proc/self/mountinfo` bzw. OS-Schnittstellen, ob das Datenverzeichnis auf einem verschlüsselten Blockdevice liegt. Ist dies nicht verifizierbar, wird eine Systemwarnung emittiert. |
| **2** | **Swap-Speicher** | OS-Swap muss entweder vollständig deaktiviert (`swapoff -a`) oder mittels eines flüchtigen, bei jedem Boot neu generierten Schlüssels verschlüsselt sein (`dm-crypt` mit `/dev/urandom`). | **Beobachtet:** Pre-Flight-Checks lesen `/proc/swaps` / OS-Metrics. Aktivierter, unverschlüsselter Swap führt zu einer Warnung / Policy-Verletzung. |
| **3** | **Core Dumps & Crash Dumps** | Anwendungs- und Kernel-Core-Dumps müssen systemweit deaktiviert sein (`ulimit -c 0`, `coredump.conf: Storage=none`), um das Auslesen von Schlüsselmaterial (`Zeroize`) aus Process-Memory-Dumps zu verhindern. | **Beobachtet:** Contextra prüft die RLIMIT_CORE Ressource-Limits beim Start. |
| **4** | **Hibernation / Sleep States** | Ruhezustand (*Suspend-to-Disk / Hibernation*) muss deaktiviert sein, damit RAM-Inhalte mit KEK/DEK-Schlüsselmaterial nicht unverschlüsselt auf die Disk geschrieben werden. | **Nicht prüfbar:** Liegt in der Verantwortung der OS-Betriebsführung / Systemd-Konfiguration. |
| **5** | **Hypervisor-Snapshots & Live-Migration** | VM-Snapshots, RAM-Captures und Live-Migrationen bergen das Restrisiko, flüchtige Schlüsselzustände einzufrieren oder auf Sekundärspeicher zu kopieren. | **Nicht prüfbar:** Aus der Gast-VM heraus nicht zuverlässig feststellbar; Infrastruktur-Restrisiko. |
| **6** | **CoW-Dateisysteme & Snapshots** | Bei Copy-on-Write-Dateisystemen (z. B. Btrfs, ZFS) führen In-Place-Scrubbing-Versuche zu neuen Chunks statt Überschreibung. Snapshots verhindern die Zerstörung des physischen Blocks. | **Beobachtet:** `contextra-durable-fs` liest `/proc/self/mountinfo` sowie Btrfs/ZFS-IOCTLs aus. Erkennt Contextra ein CoW-Dateisystem oder aktive FS-Snapshots, stuft sie physisches Overwriting ab und erzwingt Crypto-Erase als primären Pfad. |
| **7** | **SED-Warnung (Self-Encrypting Deception)** | Hardware-SEDs (Self-Encrypting Drives) versprechen "Instant Sanitize", verschleiern jedoch oft Firmware-Bugs, fehlende Entropie oder unvollständige Schlüssellöschung. | **Nicht prüfbar:** Contextra verlässt sich **niemals** auf SED-Hardwarelöschung allein, sondern erzwingt stets FDE auf Betriebssystemebene und softwareseitiges Crypto-Key-Shredding. |
| **8** | **S3 Delete Marker & Object-Lock** | Bei Nutzung von S3/S3-kompatiblem Objekt-Speicher führen einfache Delete-Requests nur zu *Delete Markern*; versionierte Objekte und *Object-Lock (Governance/Compliance)* verhindern die Löschung. | **Beobachtet / Protokolliert:** Contextra-S3-Export-Module prüfen Object-Lock-Header und emittieren explizite Hinweise, falls Löschaufrufe aufgrund von WORM-Policies blockiert werden. |
| **9** | **KMS-Wartefristen (AWS / Google Cloud KMS)** | Externe Cloud-KMS-Dienste erzwingen bei Schlüssellöschung Sicherheits-Wartefristen (AWS KMS: 7–30 Tage; Google Cloud KMS: 24 Stunden bis 120 Tage; typischer Branchenstandard: 45 Tage bis zur endgültigen physikalischen Entfernung). | **Beobachtet / Protokolliert:** Contextra vermerkt ausstehende KMS-Schlüssel-Deaktivierungen im `ErasureLedger` mit Status `PendingDestruction` und protokolliert die verbleibende Wartefrist. |

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
