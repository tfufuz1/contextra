# Rechtsmatrix zur Löschung (LEGAL_MATRIX_ERASURE.md)

**Hinweis:** Vorlage, keine Rechtsberatung.

---

### Rechtlicher Vorbehalt & Zitatbehandlung

> **WICHTIGER HINWEIS:** Dieses Dokument stellt das Ergebnis einer technischen und rechtlichen Recherche dar und bildet eine technische Entscheidungsgrundlage. Es stellt **keine Rechtsberatung** dar und ersetzt keine juristische Prüfung im Einzelfall. Zur Vermeidung von Urheberrechts- und Lizenzkonflikten enthalten alle nachfolgenden Referenzen auf Gesetzestexte, Verordnungen, Rechtsprechung, EDPB-Leitlinien und Industrienormen **ausschließlich normativ-systematische Querverweise** und Zusammenfassungen technischer Anforderungen; es werden keine geschützten Original-Wortlaute wiedergegeben.

---

### 1. Juristische Klarstellungen & Kernannahmen

1. **Rechtsgrundlage der Nachweispflicht (Art. 17 vs. Art. 5 Abs. 2 DSGVO):**
   Artikel 17 DSGVO ("Recht auf Löschung") gewährt der betroffenen Person das materielle Recht auf Löschung und verpflichtet den Verantwortlichen zur unverzüglichen Löschung personenbezogener Daten, schreibt jedoch weder ein spezifisches technisches Löschverfahren noch ein bestimmtes Nachweisformat vor. Die Pflicht zur gerichtsfesten und auditierbaren Dokumentation der vollzogenen Löschung ergibt sich systematisch aus **Art. 5 Abs. 2 DSGVO (Rechenschaftspflicht / Accountability)** in Verbindung mit Art. 24 Abs. 1 DSGVO.

2. **Kryptographische Signaturen, Beweiskraft und eIDAS (Beweiswürdigung vs. Ed25519):**
   Die im Contextra-System erzeugten kryptographischen Löschnachweise (`DeletionProof`) basieren auf Ed25519-Signaturen.
   * **eIDAS / QES:** Eine Ed25519-Signatur stellt eine fortgeschrittene elektronische Signatur dar, ist jedoch **keine qualifizierte elektronische Signatur (QES)** gemäß Art. 3 Nr. 12 i. V. m. Art. 28 eIDAS-Verordnung (EU) Nr. 910/2014.
   * **Elektronische Beweisführung:** Ein automatischer Anscheinsbeweis greift nur bei öffentlichen Dokumenten oder privaten Dokumenten mit qualifizierter elektronischer Signatur. Ed25519-signierte Löschnachweise unterliegen der freien Beweiswürdigung (§ 286 ZPO).
   * **Zeitstempel:** Ein von einer vertrauenswürdigen Zertifizierungsstelle (TSA) ausgestellter **qualifizierter elektronischer Zeitstempel (Art. 42 eIDAS)** bietet gegenüber rein systeminternen Systemzeit-Stempeln eine erheblich höhere Beweiskraft gegen nachträgliche Manipulation und Rückdatierung.

3. **Rechtlicher Status von Crypto-Erase (Verifikationsstufe V3):**
   Ob das Unbrauchbarmachen von Daten durch die unwiderrufliche Vernichtung des Verschlüsselungsschlüssels (*Crypto-Erase / Key Shredding*) rechtlich vollumfänglich als Löschung im Sinne von Art. 17 DSGVO anerkannt wird, ist bisher **juristisch offen und ungeklärt (Verifikationsstufe V3)**. Weder der Europäische Datenschutzausschuss (EDPB) noch der EuGH oder deutsche Landesdatenschutzbeauftragte haben eine bedingungslose Gleichstellung von Crypto-Erase mit physikalischer Überschreibung oder Vernichtung bestätigt, solange der Chiffretext auf dem Speichermedium verbleibt. Crypto-Erase gilt in der Praxis jedoch bei ordnungsgemäßer Schlüssellänge und CSPRNG-Schlüsselerzeugung als primäres technisches Mittel für unumkehrbaren Datenzugriffsverlust auf CoW- und Modern-Flash-Speichern.

---

### 2. Normative Rechts- und Technikmatrix zur Löschung

**Verifikationsstufen:**
* **V1 (Deterministisch verifiziert):** Kryptographisch oder bitgenau auf der lokalen Instanz prüfbar (z. B. Proof-Signatur, Zero-Key-Verification, MemTable-Scanner).
* **V2 (Beobachtet / Protokolliert):** Betrieblich/systemisch beobachtet, aber abhängig von Umgebungsbedingungen (z. B. fsync-Bestätigung, POSIX-Unlink, WAL-Retirement).
* **V3 (Rechtlich/Betrieblich unvollständig / Restrisiko):** Technisch auf Ebene des Anwendungs-Nodes nicht garantierbar (z. B. SSD-Wear-Leveling, Hardware-CoW, Cloud-KMS-Wartefristen, rechtliche Grauzone).

| Anforderung / Norm | Technikbaustein (Contextra) | Nachweisart | Verifikationsstufe | Lücke / Restrisiko |
| :--- | :--- | :--- | :---: | :--- |
| **DSGVO Art. 5 Abs. 1 lit. e**<br>*(Speicherbegrenzung)* | TTL-Reaper, Expiry-Engine, Purge-Compaction in `contextra-store` | Expiry-Log, Manifest-Transaktionshistorie | **V1** | Verzögerung zwischen TTL-Ablauf und nächstem Compaction-Durchlauf (Purge-Cycle). |
| **DSGVO Art. 5 Abs. 2**<br>*(Rechenschaftspflicht)* | `DeletionProof` v3/v4 in `contextra-crypto`, `ErasureLedger` in `contextra-store` | Ed25519-signierter Löschnachweis, Audit-Chain-Anchor | **V1** | Nachweis belegt logische Löschung/Key-Shredding; garantiert keine physikalische Zerstörung der Flash-Zellen (siehe V3). |
| **DSGVO Art. 17 (Abs. 1–3)**<br>*(Recht auf Löschung)* | `Collection::erase_document`, `ErasureFence`, Crypto-Shredding via `RevocationLog` | Zweiseitiger 2-Stage-Nachweis (`DpiPurgeDriver`, Stage 1 Logical, Stage 2 Physical) | **V1 / V3** | Crypto-Erase rechtlich im Graubereich (V3); Flash-Wear-Leveling entzieht sich der Node-Kontrolle. |
| **DSGVO Art. 19**<br>*(Mitteilungspflicht)* | MCP-Tooling, Audit-Export Egress Verification (`contextra-audit-export`) | Egress-Verifikationsbericht, Downstream-Event-Receipt | **V2** | Erfordert aktive Mitwirkung/Anbindung externer Downstream-Systeme. |
| **DSGVO Art. 25**<br>*(Privacy by Design / Default)* | Scoped Storage Isolation, Tenant-Segregation, Automatic Memory Zeroization (`Zeroize`) | Compile-Time `#![forbid(unsafe_code)]`, Type-Safe Identifiers | **V1** | Abhängig von korrekt konfigurierten Tenant-Policies bei Systemstart. |
| **DSGVO Art. 28 Abs. 3 lit. g**<br>*(Auftragsverarbeiter: Löschung/Rückgabe)* | `DpiDeletionRuntime`, Purge-Orchestrator, Tenant-Segment-Shredding | Tenant-Shredding-Zertifikat, Ledger-Scrubbing-Report | **V2** | Backup-Retention-Zeiten außerhalb der direkten Instanzsteuerung müssen vertraglich geregelt sein. |
| **DSGVO Art. 32**<br>*(Sicherheit der Verarbeitung)* | AES-256-GCM-SIV Enklaven/Segment-Cipher, Key-Manager, FDE-Requirement | In-Memory `Zeroize`, HMAC-WAL-Integritätsprüfung | **V1** | OS-Swap oder Core Dumps können bei unzureichender Betriebskonfiguration Schlüssel leaken. |
| **Anscheinsbeweis / ZPO**<br>*(Beweiskraft elektronischer Dokumente)* | `DeletionProof` v3/v4 mit Ed25519-Signatur und Invarianten-Alignment | Kryptographisches JSON-Proof mit Signatur-Prüfsumme | **V2** | Keines der Formate ist eine QES; unterliegt der freien Beweiswürdigung (§ 286 ZPO). |
| **Art. 42 eIDAS**<br>*(Qualifizierter elektronischer Zeitstempel)* | RFC 3161 Time-Stamp-Protocol Adapter Slot (optional external integration) | TSA Token Response (Qualifizierter Zeitstempel) | **V2** | Standardmäßig wird die Systemzeit genutzt; RFC 3161 TSA-Anbindung erfordert externe Netzdienste. |
| **EuGH C-413/23 P**<br>*(Relativer Ansatz bei Personenbezug)* | Key-Shredding der Tenant-/Dokument-DEK in `contextra-crypto` | Key-Revocation-Proof, Entropy-Destruction-Witness | **V1** | Nach Key-Shredding ist Chiffretext für Dritte unlesbar (relativer Personenbezug aufgehoben). |
| **EDPB Leitlinien 02/2025 (v2.0, 07.07.2026)**<br>*(Anonymisierung & Löschkonzepte)* | K-Anonymity Guard, Pseudonymisation Engine, Crypto-Shredding | Anonymity-Audit-Log, Irreversibility Witness | **V2** | Restrisiko von Re-Identifikation bei Verknüpfung mit externen Zusatzdatenbeständen. |
| **EDPB CEF 2025**<br>*(Coordinated Enforcement: Backups & Löschnachweise)* | Automated Proof-Issuance, Backup-Filter Hooks (`DpiRestoreFilter`) | Systematischer automatisierter Löschbericht, Restore-Block-Rule | **V1** | Ältere Cold-Storage-Backups verlangen verzögerten Ledger-Abgleich beim Wiedereinspielen. |
| **SDM-Baustein 60**<br>*(Standard-Datenschutzmodell: Löschung)* | Systematisches Löschkonzept, Expiry-Rules, Stage-1/Stage-2 Processing | SDM-Löschprotokoll, Policy-Compliance-Report | **V2** | Erfordert kontinuierliche betriebliche Abnahme der Löschklassen und Fristen. |
| **DIN EN ISO/IEC 27555:2025-09**<br>*(Guidelines for data deletion, ers. DIN 66398)* | Löschklassen-Engine, Lifecycle-Reaper, Granular Purge Rules | Deletion-Governance-Log, ISO 27555 Compliance Matrix | **V2** | Zuordnung von Dokumentklassen zu Aufbewahrungsfristen obliegt der Anwendungskonfiguration. |
| **NIST SP 800-88 Rev. 2**<br>*(Guidelines for Media Sanitization)* | `scrub_and_remove` in `contextra-durable-fs`, Crypto-Clear Layer | ScrubReceipt (Bytes overwrite count), Key-Zeroization | **V2 / V3** | Clear/Purge auf logischer Ebene verifiziert; Hardware-Destroy (Physikalische Zerstörung) unmöglich. |

---
