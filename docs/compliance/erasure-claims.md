# Claim-Matrix für Löschungsmodi (Erasure Claims Matrix)

> **Rechtlicher Pflicht-Hinweis:**
> Alle rechtlichen Bewertungen und Normbezüge in diesem Dokument stellen das Ergebnis einer technischen und datenschutzrechtlichen **Recherche** dar und sind **keine Rechtsberatung**. Vor der Verwendung von Außenformulierungen, Verträgen, Zertifizierungen oder Produktversprechen in rechtsverbindlichen Kontexten ist zwingend eine **Freigabe durch den Datenschutzbeauftragten (DSB) und die Rechtsabteilung** einzuholen. `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) erforderlich.]`

---

## 1. Kontext und Zielsetzung

Das Versprechen *"gelöscht heißt gelöscht"* ist als absolutes Schutzversprechen technisch und rechtlich nicht haltbar. Die Datenschutz-Grundverordnung (DSGVO, VO (EU) 2016/679) verlangt Löschung nach dem Stand der Technik sowie Nachweisbarkeit und Rechenschaft (Art. 5 Abs. 2, Art. 17, Art. 24, Art. 25, Art. 32 DSGVO).

Dieses Dokument definiert eine verbindliche Claim-Matrix für die drei Stufenmodi der Contextra-Engine (`Off`, `Assured`, `Attested`). Sie legt fest, was das System technisch leistet, was nicht garantiert wird, welche Restrisiken bestehen und welche Formulierungen in der Außenkommunikation zulässig bzw. verboten sind. `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) erforderlich.]`

---

## 2. Claim-Matrix für Löschungsmodi

| Modus | Was geschieht technisch | Was nicht garantiert ist | Restrisiken (SSD-Remanenz, Backups, Prozessspeicher, Kopien) | Zulässige Außenformulierung | Verweis auf Code |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Off** | Standardmäßige Anwendungs-Löschung (Soft-Delete oder logisches Entfernen von Datensätzen ohne physischen Bereinigungslauf oder kryptografische Löschnachweise). | Keine Garantie für physische Löschung in LSM-Trees, SSTables, WAL-Dateien, Vektor-Indizes (HNSW) oder Caches. Kein Löschnachweis, kein Audit-Ledger. | **Sehr hoch:** Daten verbleiben auf unbestimmte Zeit in unkompaktierten SSTables, WAL-Segmenten, OS-Puffern, RAM, Wear-Leveling-Blöcken von SSDs und Backups. | *"Standard-Löschung auf Anwendungsebene ohne erweiterte physische oder kryptografische Löschprüfungen."* | [`crates/contextra-ports/src/erasure.rs`](../../crates/contextra-ports/src/erasure.rs) (`ErasureMode::Off`), [`crates/contextra/src/performance_profile.rs`](../../crates/contextra/src/performance_profile.rs), [`crates/contextra-engine/src/lib.rs`](../../crates/contextra-engine/src/lib.rs) |
| **Assured** | Dauerhafte Eintragung des Löschauftrags (`ErasureIntent`) im Ledger inkl. synchronem Disk-Flush (`fsync`), physischer Hintergrund-Purge (`DpiPurgeDriver` / `run_physical`), Ausführung des Drain-Fence-Protokolls B und Tombstone-Installation. | Keine Generierung von Drittpartei-verifizierbaren, digital signierten `DeletionProof` v4 Nachweisen; keine Überschreibung physikalischer Speicherzellen auf Hardware-Ebene (SSD Wear-Leveling). | **Mittel bis hoch:** Flash-Speicher-Remanenz (SSD Controller FTL Wear-Leveling), OS-Dateisystem-Journale, kalte Offline-Backups vor dem Löschzeitpunkt, flüchtiger Prozessspeicher (RAM) vor der Garbage Collection / Zeroization, externe Anwendungskopien außerhalb der Engine. | *"Strukturierte, systemische Löschung auf Engine-Ebene mit dauerhaftem Intent-Ledger und physischem Speicheraufräumprozess nach Stand der Technik."* | [`crates/contextra-ports/src/erasure.rs`](../../crates/contextra-ports/src/erasure.rs) (`ErasureMode::Assured`), [`crates/contextra-engine/src/deletion/purge_driver.rs`](../../crates/contextra-engine/src/deletion/purge_driver.rs), [`crates/contextra-durable-fs/src/lib.rs`](../../crates/contextra-durable-fs/src/lib.rs) |
| **Attested** | Echte physische Prefix-Scans über alle aktiven Speicherschichten (LSM-Memtable, SSTables, WAL, TextIndex, Relations), Ausstellung unfehlbarer `LayerWitness`-Instanzen, Erzeugung eines kanonischen TLV-kodierten, mit Ed25519 signierten `DeletionProof` v4 mit Blake3-Footprint-Hashes und Eintragen im Audit-Trail. | Keine Qualifizierte Elektronische Signatur (QES) i.S.v. eIDAS / § 371a ZPO; keine Vernichtung physikalischer NAND-Hardware-Blöcke; keine Unkenntlichmachung außerhalb der Systemgrenze (Offline-Backups, KI-Modellgewichte nach Fine-Tuning). | **Gering bis mittel:** SSD Wear-Leveling-Remanenz auf Controllerebene, historische unveränderliche Backup-Medien (WORM/Tape), flüchtige RAM-Reste vor Prozessbeendigung, Datenkopien außerhalb der Contextra-Systemgrenzen. | *"Kryptografisch nachweisbare Löschung auf Systemebene nach Stand der Technik mit mathematischem Audit-Nachweis über durchsuchte Speicher-Schichten."* | [`crates/contextra-ports/src/erasure.rs`](../../crates/contextra-ports/src/erasure.rs) (`ErasureMode::Attested`), [`crates/contextra-crypto/src/deletion_proof/proof_v4.rs`](../../crates/contextra-crypto/src/deletion_proof/proof_v4.rs), [`crates/contextra-crypto/src/deletion_proof/canon.rs`](../../crates/contextra-crypto/src/deletion_proof/canon.rs), [`crates/contextra-engine/src/deletion/relations_witness.rs`](../../crates/contextra-engine/src/deletion/relations_witness.rs), [`crates/contextra-engine/src/deletion/text_witness.rs`](../../crates/contextra-engine/src/deletion/text_witness.rs), [`crates/contextra-store/src/wal/witness_scan.rs`](../../crates/contextra-store/src/wal/witness_scan.rs) |

---

## 3. Offene rechtliche Fragen

* `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) / Rechtsabteilung erforderlich.]`

Die folgenden fünf Kernpunkte wurden im Rahmen der rechtlichen Vorrecherche identifiziert. Sie stellen offene Rechts- und Anwendungsfragen dar, für die im Einzelfall eine juristische Bewertung und Abstimmung mit dem Verantwortlichen einzuholen ist:

1. **Ed25519-Signatur vs. Qualifizierte Elektronische Signatur (QES):**
   Eine automatisiert erzeugte Ed25519-Signatur stellt **keine qualifizierte elektronische Signatur (QES)** im Sinne der eIDAS-Verordnung (VO (EU) Nr. 910/2014) dar. Dementsprechend greift die gesetzliche Vermutung der Echtheit und Beweiskraft gemäß **§ 371a ZPO** (Zivilprozessordnung) im gerichtlichen Verfahren **nicht**. Der `DeletionProof` dient als technisches Auditinstrument nach Art. 5 Abs. 2 DSGVO, nicht als öffentlicher Urkundenbeweis. `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) erforderlich.]`

2. **EuGH C-413/23 P (Urteil vom 04.09.2025) – Pseudonymisierung und Personenbezug:**
   Nach der Rechtsprechung des EuGH im Verfahren C-413/23 P sind pseudonymisierte Daten nicht für jeden beliebigen Empfänger personenbezogen. Für den Verantwortlichen, der über den Zuordnungsschlüssel verfügt, behalten sie jedoch ihren Personenbezug. Das Urteil erging zur Verordnung (EU) 2018/1725. Die konkrete Übertragbarkeit der Grundsätze auf die DSGVO (VO (EU) 2016/679) und auf kryptografisches Key-Shredding / Hash-Verfahren ist rechtlich offen und muss im Einzelfall durch den DSB geprüft werden. `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) erforderlich.]`

3. **Begrifflichkeit "Löschen" gemäß Aufsichtsbehörden (BayLfD):**
   Der Bayerische Landesbeauftragte für den Datenschutz (BayLfD) definiert Löschen in seinem Arbeitspapier *"Löschung oder Archivierung?"* (Stand 12/2022) als Unkenntlichmachen in der Weise, dass eine Wiederherstellung nicht mehr möglich ist. Beim Crypto-Shredding (Vernichtung von Schlüsselmaterial bei Verbleib verschlüsselter Chiffretexte) ist juristisch zu klären, ob die technische Unmöglichkeit der Entschlüsselung nach aktuellem Stand der Technik der behördlichen Anforderung an die Löschung vollumfänglich genügt. `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) erforderlich.]`

4. **Personenbezug und Aufbewahrung des Löschnachweises:**
   Ein Löschnachweis (`DeletionProof` / Audit-Eintrag) enthält technische Meta-Daten (Kryptografische Hashes, Dokumenten-IDs, Zeitstempel, Scope-Identifikatoren). Diese Daten stellen pseudonyme Informationen dar und unterliegen selbst der DSGVO. Der Löschnachweis benötigt daher eine eigene Rechtsgrundlage (z. B. Art. 6 Abs. 1 lit. c / lit. f DSGVO i. V. m. Art. 5 Abs. 2 DSGVO Rechenschaftspflicht) sowie eine definierte Lösch- und Aufbewahrungsfrist. `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) erforderlich.]`

5. **Spannungsverhältnis Anti-Resurrection-Ledger vs. Art. 17 DSGVO:**
   Der Betrieb eines Anti-Resurrection-Ledgers (Sperrliste / Revocation-Log zur Verhinderung des unbeabsichtigten Re-Imports gelöschter Daten aus historischen Backups) erfordert das dauerhafte Speichern von Identifikatoren oder Hashwerten gelöschter Objekte. Dies steht in einem strukturellen Spannungsverhältnis zu Art. 17 DSGVO (Recht auf Vergessenwerden) und erfordert eine dokumentierte Schutzbedarfs- und Interessenabwägung (Art. 6 Abs. 1 lit. f DSGVO). `[Recherche, keine Rechtsberatung. Freigabe durch Datenschutzbeauftragten (DSB) erforderlich.]`

---

## 4. Verbotene Formulierungen

Die folgenden Begriffe und Werbeaussagen sind in der Produkt-Dokumentation, im Marketing, in Vertriebsunterlagen und in Kundenverträgen **ausdrücklich untersagt**:

- ❌ **"gelöscht heißt gelöscht"** (ohne Einschränkung auf den technischen Gültigkeitsbereich, Systemgrenzen und Restrisiken).
- ❌ **"rechtsverbindlich"** oder **"gerichtsverwertbar"** (da keine qualifizierte elektronische Signatur gemäß eIDAS / § 371a ZPO vorliegt).
- ❌ **"beweiskräftig"** im juristischen Sinne (für Nachweise ohne qualifizierte elektronische Signatur nach eIDAS).
- ❌ **"garantiert"** oder **"absolute Garantie"** in Bezug auf die vollständige Abwesenheit von Restrisiken (wie SSD-Wear-Leveling-Remanenz, OS-Journale, RAM-Reste oder historische Backups).

---

## 5. Normen und Referenzdokumente

### Geprüfte Quellen & Normen
* `[Geprüft]` **DSGVO (VO (EU) 2016/679):**
  - **Art. 5 Abs. 2:** Rechenschaftspflicht des Verantwortlichen.
  - **Art. 17:** Recht auf Löschung ("Recht auf Vergessenwerden").
  - **Art. 24:** Verantwortung des für die Verarbeitung Verantwortlichen.
  - **Art. 25:** Datenschutz durch Technikgestaltung und durch datenschutzfreundliche Voreinstellungen (Privacy by Design / Default).
  - **Art. 32:** Sicherheit der Verarbeitung (Stand der Technik).
* `[Geprüft]` **ZPO (Zivilprozessordnung):**
  - **§ 371a ZPO:** Beweiskraft elektronischer Dokumente mit qualifizierter elektronischer Signatur.
* `[Geprüft]` **eIDAS-Verordnung (VO (EU) Nr. 910/2014):**
  - Regelung zu elektronischen Signaturen (eIDAS / QES).
* `[Geprüft]` **EuGH C-413/23 P:**
  - Urteil vom 04.09.2025 zu Verordnung (EU) 2018/1725 (Pseudonymisierte Daten und Personenbezug).
* `[Geprüft]` **Bayerischer Landesbeauftragter für den Datenschutz (BayLfD):**
  - Arbeitspapier *"Löschung oder Archivierung?"* (Stand 12/2022).

### Zu prüfende Standards & Referenzen
* `[Zu prüfen]` **NIST SP 800-88 Rev. 1:** *Guidelines for Media Sanitization* (Anwendbarkeit auf SSD-Hardware und Crypto-Shredding in Cloud- / Container-Umgebungen).
* `[Zu prüfen]` **DIN 66399:** *Maschinen / Datenträgervernichtung* (Anwendbarkeit bei physischer Speichermedienvernichtung).
* `[Zu prüfen]` **ISO/IEC 27001 / ISO/IEC 27701:** *Information security management / Privacy information management* (Einbettung der Löschnachweise in ISMS/PIMS-Prozesse).

---

## 6. Geprüfte Fundstellen in `docs/spec/`

Die folgenden Fundstellen in der technischen Spezifikation (`docs/spec/`) bezüglich Löschung, Löschbeweisen und Nachweisen wurden verifiziert:

- `docs/spec/00-meta-workspace-layout.md:237`
- `docs/spec/00a-titel-und-einleitung.md:36, 62, 136, 170, 171`
- `docs/spec/00b-teil-a-stabilisierungsauftrag.md:19, 50`
- `docs/spec/00e-teil-a4-architekten-review-sota.md:143`
- `docs/spec/02-produktvision-und-nicht-ziele.md:34`
- `docs/spec/03-architekturprinzipien.md:23, 36`
- `docs/spec/04-crate-graph.md:36, 117, 159, 165`
- `docs/spec/05-speicherschicht.md:115, 290, 295, 345, 400, 403`
- `docs/spec/06-wissensgraph-datenmodell-b.md:90, 102, 108, 122`
- `docs/spec/09-inferenz-kv-cache-ipc.md:46`
- `docs/spec/10-sicherheits-und-datenschutzmodell.md:9`
- `docs/spec/12-flatbuffers-schema.md:17`
- `docs/spec/16-abnahmekriterien.md:15, 33, 45, 53`
- `docs/spec/18-gesamtroadmap.md:16`
- `docs/spec/20-migrationsplan-und-adr.md:28, 30`
- `docs/spec/21-sota-algorithmen-erweiterung-a.md:120, 243`
- `docs/spec/21-sota-algorithmen-erweiterung-b.md:86, 114, 167, 207, 217, 224`
- `docs/spec/90-anhang-b-begruendungen-a.md:49, 104, 138, 140, 146, 148, 168, 176, 183, 208, 233, 241, 262, 307, 332, 361, 405`
- `docs/spec/90-anhang-b-begruendungen-b.md:28, 53, 86, 131, 156, 170, 199, 233, 256, 270, 290, 338, 352`
- `docs/spec/91-anhang-c-aenderungsprotokoll-v2-1.md:5, 32`
- `docs/spec/92-anhang-d-aenderungsprotokoll-v4.md:5, 32, 79`
- `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:35, 125, 154, 185, 234, 276, 291, 394, 418, 423, 425, 482, 502, 522, 544, 654, 770, 842, 910, 1433, 1441, 1442, 1446, 1447, 1448, 1449, 1471, 1484, 2000, 2265, 2577, 3831, 4009, 4067, 4077, 4115, 4126`
- `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md:35, 125, 154, 185, 234, 276, 291, 394, 418, 423, 425, 482, 502, 522, 544, 654, 770, 842, 910, 1433, 1441, 1442, 1446, 1447, 1448, 1449, 1471, 1484, 2000, 2265, 2577, 3831, 4009, 4067, 4077, 4115, 4126`
- `docs/spec/INDEX.md:57, 58`
