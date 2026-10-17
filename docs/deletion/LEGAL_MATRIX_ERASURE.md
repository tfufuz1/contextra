# Rechtsmatrix zur Löschung (LEGAL_MATRIX_ERASURE.md)

**Hinweis:** Vorlage, keine Rechtsberatung.

---

### Implementierungs- & Verifikationsstatus

| Baustein / Bereich | Status | Verifikationsstufe | Anmerkung / Code-Anker |
| :--- | :---: | :---: | :--- |
| **TTL & Expiry-Engine** | Implementiert | **V1** | `crates/contextra-engine/src/collection/maintenance.rs:24` |
| **DeletionProof v3/v4 & ErasureLedger** | Implementiert | **V1** | `crates/contextra-crypto/src/deletion_proof/proof_v4.rs:58`, `crates/contextra-store/src/lsm/commit.rs:1` |
| **Purge-Driver & Logical Erasure** | Implementiert | **V1** | `crates/contextra-engine/src/deletion/purge_driver.rs:352`, `crates/contextra-store/src/lsm/fence.rs:339` |
| **Crypto-Erase (Key Shredding)** | Implementiert (technisch) / Rechtlich offen | **V3** | `crates/contextra-crypto/src/revocation_log.rs:285`, `crates/contextra-crypto/src/kms.rs:1` |
| **Scoped Storage & Zeroize** | Implementiert | **V1** | `crates/contextra-crypto/src/secure_buf.rs:1`, `crates/contextra-py/src/bindings/db.rs:1` |
| **Scrub & Physical Overwrite** | Implementiert (Best-Effort POSIX) | **V2** | `crates/contextra-durable-fs/src/lib.rs:364` |
| **RFC 3161 Zeitstempel / RFC 9162 Log** | Implementiert | **V2** | `crates/contextra-crypto/src/deletion_proof/timestamp_anchor.rs:391` |
| **OS-Umgebungsbeobachtung (Mount, Swap, CoreDump)** | Implementiert | **V2** | `crates/contextra-sys/src/env_observe.rs:418`, `crates/contextra-sys/src/hardening.rs:36` |
| **K-Anonymity Guard & Pseudonymisation Engine** | Geplant | **—** | Nicht in Produktion implementiert |
| **Entropy-Destruction-Witness & Anonymity-Audit-Log** | Geplant | **—** | Nicht in Produktion implementiert |
| **Downstream Egress Notification (Art. 19)** | Teilweise / Geplant | **V2** | Egress-Verifikation in `contextra-audit-export`, automatisierte Downstream-Integration geplant |

---

### Rechtlicher Vorbehalt & Zitatbehandlung

> **WICHTIGER HINWEIS:** Dieses Dokument stellt das Ergebnis einer technischen und rechtlichen Recherche dar und bildet eine technische Entscheidungsgrundlage. Es stellt **keine Rechtsberatung** dar und ersetzt keine juristische Prüfung im Einzelfall. Zur Vermeidung von Urheberrechts- und Lizenzkonflikten enthalten alle nachfolgenden Referenzen auf Gesetzestexte, Verordnungen, Rechtsprechung, EDPB-Leitlinien und Industrienormen **ausschließlich normativ-systematische Querverweise** und Zusammenfassungen technischer Anforderungen; es werden keine geschützten Original-Wortlaute wiedergegeben.

---

### 1. Juristische Klarstellungen & Kernannahmen

1. **Rechtsgrundlage der Nachweispflicht (Art. 17 vs. Art. 5 Abs. 2 DSGVO):**
   Artikel 17 DSGVO ("Recht auf Löschung") gewährt der betroffenen Person das materielle Recht auf Löschung und verpflichtet den Verantwortlichen zur unverzüglichen Löschung personenbezogener Daten, schreibt jedoch weder ein spezifisches technisches Löschverfahren noch ein bestimmtes Nachweisformat vor. Die Pflicht zur auditierbaren Dokumentation der vollzogenen Löschung ergibt sich systematisch aus **Art. 5 Abs. 2 DSGVO (Rechenschaftspflicht / Accountability)** in Verbindung mit Art. 24 Abs. 1 DSGVO.

2. **Kryptographische Signaturen, Beweiskraft und eIDAS (Beweiswürdigung vs. Ed25519):**
   Die im Contextra-System erzeugten kryptographischen Löschnachweise (`DeletionProof`, `crates/contextra-crypto/src/deletion_proof/proof_v4.rs:58`) basieren auf Ed25519-Signaturen.
   * **eIDAS / QES:** Eine Ed25519-Signatur stellt eine fortgeschrittene elektronische Signatur dar, ist jedoch **keine qualifizierte elektronische Signatur (QES)** gemäß Art. 3 Nr. 12 i. V. m. Art. 28 eIDAS-Verordnung (EU) Nr. 910/2014.
   * **Elektronische Beweisführung:** Ein automatischer Anscheinsbeweis greift nur bei öffentlichen Dokumenten oder privaten Dokumenten mit qualifizierter elektronischer Signatur. Ed25519-signierte Löschnachweise unterliegen der freien Beweiswürdigung (§ 286 ZPO).
   * **Zeitstempel:** Ein von einer vertrauenswürdigen Zertifizierungsstelle (TSA) ausgestellter **qualifizierter elektronischer Zeitstempel (Art. 42 eIDAS)** bietet gegenüber rein systeminternen Systemzeit-Stempeln eine höhere Beweiskraft gegen nachträgliche Manipulation und Rückdatierung (`crates/contextra-crypto/src/deletion_proof/timestamp_anchor.rs:391`).

3. **Rechtlicher Status von Crypto-Erase (Verifikationsstufe V3):**
   Ob das Unbrauchbarmachen von Daten durch die unwiderrufliche Vernichtung des Verschlüsselungsschlüssels (*Crypto-Erase / Key Shredding*) rechtlich vollumfänglich als Löschung im Sinne von Art. 17 DSGVO anerkannt wird, ist bisher **juristisch offen und ungeklärt (Verifikationsstufe V3)** (Primärquelle: BSI/Datenschutzkonferenz-Positionspapiere; Sekundärquelle: Branchenanalysen). Weder der Europäische Datenschutzausschuss (EDPB) noch der EuGH oder deutsche Datenschutzaufsichtsbehörden haben eine bedingungslose Gleichstellung von Crypto-Erase mit physikalischer Überschreibung oder Vernichtung bestätigt, solange der Chiffretext auf dem Speichermedium verbleibt. Crypto-Erase gilt in der Praxis bei ordnungsgemäßer Schlüssellänge und CSPRNG-Schlüsselerzeugung als primäres technisches Mittel für unumkehrbaren Datenzugriffsverlust auf Copy-on-Write- und Modern-Flash-Speichern.

4. **Rechtsprechung des EuGH zu pseudonymen Daten (EuGH C-413/23 P, SRB / EDPS, 04.09.2025):**
   Das EuGH-Urteil C-413/23 P (EDPS/SRB vom 04.09.2025; Primärquelle: Urteil des EuGH) betrifft den **relativen Personenbezug pseudonymisierter Daten aus Sicht des Datenempfängers**. Der EuGH stellte klar, dass Daten für einen Empfänger nicht personenbezogen sind, wenn dieser nicht über die rechtlichen und tatsächlichen Mittel verfügt, die betroffene Person zu re-identifizieren. Das Urteil behandelt **nicht** das Key Shredding oder Crypto-Erase als Löschmethode nach Art. 17 DSGVO, liefert jedoch die dogmatische Grundlage dafür, dass Chiffretexte ohne verfügbaren Schlüssel für Stellen ohne Re-Identifikationsmöglichkeit ihren Personenbezug verlieren können.

5. **EDPB-Leitlinien & CEF-Bericht 2025/2026 (Primärquelle: EDPB CEF Report, 10.02.2026):**
   Der EDPB-Bericht zur koordinierten Prüfung des Rechts auf Löschung (*Coordinated Enforcement Action / CEF Report on Right to Erasure*, verabschiedet am 10.02.2026; Primärquelle: EDPB) weist darauf hin, dass die EDPB-Leitlinien 02/2025 sich auf die Verarbeitung personenbezogener Daten in **Blockchain-Anwendungen** beziehen (*Guidelines 02/2025 on processing of personal data through blockchain applications*). Spezifische EDPB-Leitlinien zu Anonymisierungstechniken sind laut CEF-Bericht noch in Erarbeitung (*currently under development*).

---

### 2. Normative Rechts- und Technikmatrix zur Löschung

**Verifikationsstufen:**
* **V1 (Deterministisch verifiziert):** Kryptographisch oder bitgenau auf der lokalen Instanz prüfbar (z. B. Proof-Signatur, Zero-Key-Verification, MemTable-Scanner).
* **V2 (Beobachtet / Protokolliert):** Betrieblich/systemisch beobachtet, aber abhängig von Umgebungsbedingungen (z. B. fsync-Bestätigung, POSIX-Unlink, WAL-Retirement).
* **V3 (Rechtlich/Betrieblich unvollständig / Restrisiko):** Technisch auf Ebene des Anwendungs-Nodes nicht garantierbar (z. B. SSD-Wear-Leveling, Hardware-CoW, Cloud-KMS-Wartefristen, rechtliche Grauzone).

| Anforderung / Norm | Technikbaustein (Contextra) | Nachweisart | Verifikationsstufe | Lücke / Restrisiko & Status |
| :--- | :--- | :--- | :---: | :--- |
| **DSGVO Art. 5 Abs. 1 lit. e**<br>*(Speicherbegrenzung)* | TTL-Reaper, Expiry-Engine in `contextra-engine` (`crates/contextra-engine/src/collection/maintenance.rs:24`) | Expiry-Log, Manifest-Transaktionshistorie | **V1** | Verzögerung zwischen TTL-Ablauf und nächstem Compaction-Durchlauf (Purge-Cycle). |
| **DSGVO Art. 5 Abs. 2**<br>*(Rechenschaftspflicht)* | `DeletionProof` v3/v4 in `contextra-crypto` (`crates/contextra-crypto/src/deletion_proof/proof_v4.rs:58`) | Ed25519-signierter Löschnachweis | **V1** | Nachweis belegt logische Löschung/Key-Shredding; garantiert keine physikalische Zerstörung der Flash-Zellen (siehe V3). |
| **DSGVO Art. 17 (Abs. 1–3)**<br>*(Recht auf Löschung)* | `AgentMemory::forget` (`crates/contextra/src/agent_memory.rs:202`), `forget_document` (`crates/contextra-engine/src/deletion/purge_driver.rs:352`), `ErasureFence` (`crates/contextra-store/src/lsm/fence.rs:339`), Key Shredding via `RevocationLog` (`crates/contextra-crypto/src/revocation_log.rs:285`) | 2-Stage-Nachweis (`DpiPurgeDriver`, Stage 1 Interim, Stage 2 Final) | **V1 / V3** | Crypto-Erase rechtlich im Graubereich (V3); Flash-Wear-Leveling entzieht sich der Node-Kontrolle. Hinw.: `forget_document` besitzt Aufrufer im Purge-Orchestrator und Tests. |
| **DSGVO Art. 19**<br>*(Mitteilungspflicht)* | Audit-Export Egress Verification in `contextra-audit-export` (`crates/contextra-audit-export/src/lib.rs:1`) | Egress-Verifikationsbericht | **V2** | Erfordert aktive Mitwirkung/Anbindung externer Downstream-Systeme. Automatisierte Downstream-Event-Receipts sind geplantes Feature. |
| **DSGVO Art. 25**<br>*(Privacy by Design / Default)* | Scoped Storage Isolation, Tenant-Segregation, Automatic Memory Zeroization (`Zeroize`, `crates/contextra-crypto/src/secure_buf.rs:1`) | Compile-Time `#![forbid(unsafe_code)]`, Type-Safe Identifiers | **V1** | Abhängig von korrekt konfigurierten Tenant-Policies bei Systemstart. |
| **DSGVO Art. 28 Abs. 3 lit. g**<br>*(Auftragsverarbeiter: Löschung/Rückgabe)* | `DpiDeletionRuntime` (`crates/contextra-engine/src/deletion/proof_issue.rs:141`), Tenant-Key-Revocation | Tenant-Shredding-Zertifikat | **V2** | Backup-Retention-Zeiten außerhalb der direkten Instanzsteuerung müssen vertraglich geregelt sein. |
| **DSGVO Art. 32**<br>*(Sicherheit der Verarbeitung)* | Scoped Key Manager, Memory `Zeroize` (`crates/contextra-crypto/src/secure_buf.rs:1`), HMAC-WAL-Integritätsprüfung (`crates/contextra-store/src/wal/encode.rs:1`) | In-Memory `Zeroize`, HMAC-WAL-Verifikation | **V1** | OS-Swap oder Core Dumps können bei unzureichender Betriebskonfiguration Schlüssel leaken. |
| **Anscheinsbeweis / ZPO**<br>*(Beweiskraft elektronischer Dokumente)* | `DeletionProof` v3/v4 mit Ed25519-Signatur (`crates/contextra-crypto/src/deletion_proof/proof_v4.rs:58`) | Kryptographisches JSON-Proof mit Signatur-Prüfsumme | **V2** | Keines der Formate ist eine QES; unterliegt der freien Beweiswürdigung (§ 286 ZPO). |
| **Art. 42 eIDAS**<br>*(Qualifizierter elektronischer Zeitstempel)* | RFC 3161 Adapter (`crates/contextra-crypto/src/deletion_proof/timestamp_anchor.rs:391`) | TSA Token Response (Qualifizierter Zeitstempel) | **V2** | Standardmäßig wird die Systemzeit genutzt; RFC 3161 TSA-Anbindung erfordert externe Netzdienste. |
| **EuGH C-413/23 P**<br>*(Relativer Ansatz bei Personenbezug)* | Key-Shredding der Tenant-/Dokument-DEK in `contextra-crypto` (`crates/contextra-crypto/src/revocation_log.rs:285`) | Key-Revocation-Proof | **V1 / V3** | Nach Key-Shredding ist Chiffretext für Dritte unlesbar (relativer Personenbezug aufgehoben, rechtlicher Löschstatus V3). |
| **EDPB CEF 2025/2026**<br>*(Coordinated Enforcement: Backups & Löschnachweise)* | Automated Proof-Issuance (`crates/contextra-engine/src/deletion/proof_issue.rs:141`) | Systematischer automatisierter Löschbericht | **V1** | Ältere Cold-Storage-Backups verlangen verzögerten Ledger-Abgleich beim Wiedereinspielen. |
| **SDM-Baustein 60**<br>*(Standard-Datenschutzmodell: Löschung)* | Expiry-Rules, Stage-1/Stage-2 Processing (`crates/contextra-engine/src/deletion/purge_driver.rs:352`) | Löschprotokoll | **V2** | Erfordert kontinuierliche betriebliche Abnahme der Löschklassen und Fristen. |
| **DIN EN ISO/IEC 27555:2025-09**<br>*(Richtlinien zur Datenlöschung; Nachfolger DIN 66398 — zu verifizieren, Sekundärquelle)* | Lifecycle-Reaper in `contextra-engine` (`crates/contextra-engine/src/collection/maintenance.rs:24`) | Deletion-Governance-Log | **V2** | Norm-Status DIN EN ISO/IEC 27555:2025-09 als Nachfolger von DIN 66398 basiert auf Sekundärquellen und ist im Einzelfall zu verifizieren. |
| **NIST SP 800-88 Rev. 2**<br>*(Guidelines for Media Sanitization)* | `scrub_and_remove` in `contextra-durable-fs` (`crates/contextra-durable-fs/src/lib.rs:364`) | ScrubReceipt (Bytes overwrite count) | **V2 / V3** | Clear/Purge auf logischer/Best-Effort POSIX Dateisystemebene verifiziert; Hardware-Destroy (Physikalische Zerstörung) unmöglich. |
| **K-Anonymity Guard & Pseudonymisation Engine** | *Geplant* | **—** | *Geplante Komponenten, bisher nicht in Produktion implementiert.* |
| **Entropy-Destruction-Witness & Anonymity-Audit-Log** | *Geplant* | **—** | *Geplante Komponenten, bisher nicht in Produktion implementiert.* |

---
