# Offene Fragen für die juristische, steuerliche und förderrechtliche Fachberatung

**Projekt:** Contextra V5 LSM Vekordatenbank
**Stand:** Commit `c4f7299b6` (25.10.2026)
**Status:** Offene Punkte zur Vorlage bei Rechtsanwälten, Steuerberatern und Förderberatern (Abgrenzung gemäß Vorgabe)

---

## 1. Datenschutzrecht & DSGVO-Konformität (Rechtsberatung)

### 1.1 Anerkennung von Crypto-Shredding unter Art. 17 DSGVO
- **Kontext:** Contextra implementiert Scope-granulares Crypto-Shredding (ADR-N27 v5 SSTable-Format mit 72-Byte Trailer). Beim Löschen eines Scopes wird der Data Encryption Key (DEK) im `ScopeKeyring` unwiederbringlich zerstört. Die Chiffrate verbleiben bis zur nächsten Kompaktion physisch auf dem Datenträger, sind jedoch unlesbar.
- **Fragen an den Datenschutzjuristen:**
  1. Ist die Zerstörung des DEK in Verbindung mit einem nachweisbar sicheren Schlüsselspeicher (Hardware-KMS / Secure Enclave) nach aktiver Rechtsprechung des EuGH (insb. Urteil C-413/23 P zur empfängerrelativen Pseudonymisierung) und den EDPB-Leitlinien (EDPB 02/2025) rechtlich als vollständige Löschung im Sinne von Art. 17 DSGVO anerkannt?
  2. Welche Anforderungen werden an die Dokumentation der Zerstörung des Schlüssels im `RevocationLog` gestellt?

### 1.2 Formulierung rechtssicherer Löschaussagen in Audit-Zertifikaten
- **Kontext:** Das Modul `contextra-crypto` erzeugt fälschungssichere HMAC-signierte `DeletionProof` v4 Audit-Zertifikate, die die Abdeckung aller 7 rechtlichen Speicher-Layer attestieren.
- **Fragen an den Datenschutzjuristen:**
  1. Welche genauen Wortlaute müssen in den JSON-Attestaten verwendet werden, um irreführende Aussagen zu vermeiden? (z. B. Unterscheidung zwischen „physisch überschrieben“, „kryptografisch unzugänglich gemacht“ und „aus Index entfernt“).
  2. Reicht das maschinenlesbare `DeletionProof` v4 Attest gegenüber Aufsichtsbehörden als Nachweis gemäß Art. 5 Abs. 2 DSGVO (Rechenschaftspflicht) aus?

---

## 2. Urheberrecht & Lizenzierung (Rechtsberatung)

### 2.1 Dual-Licensing & Open-Source-Abgrenzung
- **Kontext:** Der Kern-Codebase soll unter der Business Source License (BSL 1.1) mit automatischem Übergang zu Apache 2.0 nach 3 Jahren bereitgestellt werden.
- **Fragen an den Urheberrechtler:**
  1. Schützt die BSL 1.1 in der gewählten Formulierung wirksam vor der unentgeltlichen kommerziellen Nutzung durch große Hyperscaler (z. B. AWS, Azure), während der Einsatz für KMU kostenfrei bleibt?
  2. Gibt es Inkompatibilitäten zwischen BSL 1.1 und den eingebundenen Rust-Crate-Lizenzen (MIT / Apache-2.0 / BSD-3-Clause)?

### 2.2 Urheberschaft bei KI-unterstützter Code-Transformation
- **Kontext:** Die Refaktorisierung und Entkopplung des Bestands-Codebase erfolgt unter intensiver Nutzung generativer KI-Systeme (LLM-Agenten).
- **Fragen an den Urheberrechtler:**
  1. Welche Nachweise müssen erbracht werden, um die vollumfängliche Urheberschaft und Verwertungsrechte am resultierenden Kernprodukt sicherzustellen?
  2. Sind spezifische Passagen im Repository-Header oder in `LICENSE` zur Deklaration von KI-gestützten Code-Anteilen erforderlich?

---

## 3. Steuerrecht & Forschungsförderung (Steuerberatung / Förderberatung)

### 3.1 Forschungszulage nach dem Forschungszulagengesetz (FZulG)
- **Kontext:** Die Entwicklung der v5-LSM-Engine mit scope-granularer AAD-Bindung und fälschungssicheren typisierten Löschbeweisen stellt eine technische Innovation im Bereich speichersicherer Datenbanken dar.
- **Fragen an den Steuerberater / Förderberater:**
  1. Erfüllen die Entwicklungsarbeiten an `contextra-store` und `contextra-crypto` die Kriterien der experimentellen Entwicklung gemäß § 2 FZulG?
  2. Welches Zeiterfassungs- und Dokumentationssystem muss ein Einzelentwickler führen, damit die Stundensätze für die Forschungszulage vom Finanzamt anerkannt werden?

### 3.2 Bilanzielle Behandlung von Software-Eigenleistungen
- **Fragen an den Steuerberater:**
  1. Bietet sich die Aktivierung der Eigenleistungen für die Softwareentwicklung nach § 248 Abs. 2 HGB an, und welche steuerlichen Konsequenzen ergeben sich daraus für das Geschäftsjahr 2026/2027?

---

## 4. Vertragsrecht & Haftung (Rechtsberatung)

### 4.1 Haftungsausschluss bei Crypto-Shredding in AGB / AVV
- **Kontext:** Wird ein DEK versehentlich zerstört oder der Schlüsselspeicher beschädigt, sind alle zugehörigen Daten unwiederbringlich verloren.
- **Fragen an den Wirtschaftsjuristen:**
  1. Wie muss der Haftungsausschluss in den Allgemeinen Geschäftsbedingungen (AGB) und Auftragsverarbeitungsverträgen (AVV) formuliert sein, um Schadensersatzansprüche wegen unwiederbringlichen Datenverlusts nach beabsichtigtem oder fehlerhaft ausgelöstem Crypto-Shredding wirksam zu begrenzen?
