# Baustein-Spezifikation: `contextra-license` (SPEC v4.0 §5, §9.2)

## 1. Identität und Rolle
* **Crate:** `contextra-license`
* **Klasse:** Baustein Klasse A (B1–B12)
* **Zweck:** Ed25519-signierte Lizenzprüfung und Feature-Ring-Autorisierung mit garantierter Non-Destructive Degradation.
* **Normativer Status:** Autarkes Baukasten-Modul (E-08), entkoppelt von monorepo-internen Hubs (`contextra-types`, `contextra-ports`).

---

## 2. Invarianten
* **INV-LICENSE-1 (Fail-Closed Verifikation):** Tritt bei der Prüfung eines geschützten Rings ein Fehler, ein Signaturfehlschlag, ein Ablauf oder ein Hash-Mismatch auf, wird fail-closed verweigert (`Err(LicenseError::...)`).
* **INV-LICENSE-2 (Fast-Ring Bypass & Non-Destructive Degradation):** Der Basis-Ring `FeatureRing::Fast` benötigt keine Lizenz und darf unter keinen Umständen gesperrt oder blockiert werden. Ein Lizenzablauf führt niemals zu Datenverlust oder Datensperre im Basis-Betrieb.
* **INV-LICENSE-3 (Kein Informationsleck):** Fehlermeldungen bei ungültiger Installation oder Signatur exponieren keine internen Hardware-IDs, Maschinendaten, geheime Hashes oder Schlüsselbits.
* **INV-LICENSE-4 (Deterministische Zeitprüfung, B11, P28):** Zeitabfragen erfolgen ausschließlich über den Crate-lokalen `Clock`-Trait oder explizite Parameter; kein `SystemTime::now()` im Kernalgorithmus.
* **INV-LICENSE-5 (Offline-Autarkie):** Sämtliche Prüfungen laufen 100% netzwerk- und cloud-unabhängig über asymmetrische Ed25519-Signaturen und BLAKE3-Hashes ab.

---

## 3. 7-stufige Verifikationskette
Jede Ringprüfung durchläuft folgende deterministische Sequenz:
1. **Schritt 1 (Fast-Ring Bypass):** Falls `requested == FeatureRing::Fast` → sofort `Ok(())`.
2. **Schritt 2 (Präsenzprüfung):** Prüfung, ob Aktivierungsdatensatz oder Lizenzpayload vorliegt.
3. **Schritt 3 (Installationsbindung):** Constant-Time-Vergleich (`constant_time_eq_32`) des 32-Byte-BLAKE3-Installations-Hashes.
4. **Schritt 4 (Kryptografische Signatur):** Ed25519-Signaturprüfung über die serialisierten Payload-Bytes.
5. **Schritt 5 (Ablaufprüfung):** Vergleich des Ablaufdatums gegen `clock.now_unix_nanos()`.
6. **Schritt 6 (Ring-Berechtigung):** Prüfung, ob der angeforderte Ring explizit in der Freigabeliste enthalten ist.
7. **Schritt 7 (Erfolgsbestätigung):** Autorisierung und Ausstellung des `AuthorizedRing`-Tokens.

---

## 4. API & Typen (B3, B4)
* **Fehlertyp:** `LicenseError` (`#[non_exhaustive]`, `thiserror::Error`), Result-Typ `Result<T>`.
* **Ringe:** `FeatureRing` (`Fast`, `Sovereign`, `Compliance`), Token `AuthorizedRing`.
* **Gates:** Trait `LicenseGate`, `OpenFastGate`, `SignedLicenseGate`.
* **Zeit:** Trait `Clock`, `SystemClock`, `FixedClock`.
* **Datensätze:** `SignedActivation`, `LicensePayload`, `SignedLicenseOptions`.

---

## 5. Transitionale Kompatibilität (Welle 1 bis Welle 6)
* Feature `ports-compat` (in `default` enthalten):
  - Bietet `From`/`Into`-Konvertierungen zu `contextra_ports::license::FeatureRing`, `LicenseError`, und `contextra_types::TenantId`.
  - Implementiert `contextra_ports::license::LicenseGate` für `SignedLicenseGate` und `OpenFastGate`.
  - Ermöglicht bestehenden Konsumenten (`contextra`) die unterbrechungsfreie Weiternutzung bis zur Welle-6-Entfernung.

---

## 6. Concurrency & Safety
* `#![forbid(unsafe_code)]`: Vollkommen unsafe-frei.
* Thread-safe (`Send + Sync` für alle Gates und Typen).
* Zustandslos (Stateless): Keine internen Mutexe oder veränderlichen Zustände während der Autorisierung.
