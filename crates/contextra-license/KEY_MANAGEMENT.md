# Schlüsselverwaltung in `contextra-license` (KEY_MANAGEMENT.md)

Dieses Dokument beschreibt die Schlüsselverwaltung, den Verifikationsablauf, das Verhalten bei Schlüsselwechseln sowie bekannte Grenzen des Crates `contextra-license`.

---

## 1. Ist-Zustand (ohne Wunschdenken)

### 1.1 Schlüsselherkunft und -injektion
In `contextra-license` gibt es **keine fest eincompilierten öffentlichen Schlüssel** und keinen internen Schlüsselspeicher (Keyring). Der öffentliche Verifikationsschlüssel (`ed25519_dalek::VerifyingKey`) muss dem Lizenz-Gate von außen übergeben werden:
- **Über `SignedActivation`:**
  - `SignedLicenseGate::from_activation(activation, verifying_key)` (`src/signed_gate.rs:509`)
  - `SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)` (`src/signed_gate.rs:515-527`)
- **Über signierte Nutzlasten (`LicensePayload`):**
  - `SignedLicenseGate::from_signed_payload(payload_bytes, signature, verifying_key)` (`src/signed_gate.rs:534-546`)
  - `SignedLicenseGate::from_signed_payload_with_clock(payload_bytes, signature, verifying_key, clock)` (`src/signed_gate.rs:550-563`)
  - `SignedLicenseGate::from_signed_payload_with_clock_and_opts(payload_bytes, signature, verifying_key, clock, options)` (`src/signed_gate.rs:574-617`)
- **Leeres Gate ohne Schlüssel:**
  - `SignedLicenseGate::no_activation()` setzt `verifying_key: None` (`src/signed_gate.rs:496-504`).

Die Signierung von Lizenzen erfolgt extern (z. B. auf einem Lizenzserver oder über `SignedActivation::create_signed(...)`, `src/signed_gate.rs:191-205`) mittels `ed25519_dalek::SigningKey`. Der private Schlüssel gelangt zu keinem Zeitpunkt in die Gate-Instanz.

### 1.2 Datenstrukturen & Fehlen einer Schlüsselkennung (Key-ID)
Die Datenstrukturen enthalten **keinerlei Schlüsselkennung** (`key_id`, Key-Fingerprint oder Algorithmen-ID):
- `SignedActivation` (`src/signed_gate.rs:159-168`):
  - `ring: FeatureRing`
  - `installation_id_hash: [u8; 32]`
  - `expires_at_unix: i64`
  - `signature: [u8; 64]`
- `LicensePayload` (`src/signed_gate.rs:264-276`):
  - `tenant_id: TenantId`
  - `allowed_rings: Vec<FeatureRing>`
  - `expires_at: Option<i64>`
  - `feature_flags: BTreeMap<String, bool>`
  - `installation_id_hash: Option<[u8; 32]>`

Ein Token verweist somit weder direkt noch indirekt auf den zu seiner Verifikation erforderlichen Schlüssel.

### 1.3 Mehrschlüssel-Fähigkeit & Rotation
- **Unterstützung mehrerer Schlüssel:** **Nicht unterstützt.** `SignedLicenseGate` speichert exakt einen optionalen Einzelschlüssel: `verifying_key: Option<VerifyingKey>` (`src/signed_gate.rs:475`).
- **Schlüsselrotation:** **Nicht unterstützt.** Es gibt keine Mechanismen für Keyrings, Schlüssel-Fallbacks, Übergangsphasen (Grace Periods) oder Überlappungen alter und neuer Schlüssel innerhalb einer Gate-Instanz.
- **Befund:** Ein Lizenz-Gate kann zu jedem Zeitpunkt genau einen öffentlichen Ed25519-Schlüssel verifizieren.

---

## 2. Verifikationsablauf & Verhalten bei unbekanntem/falschem Schlüssel

### 2.1 Verifikationskette
Die Überprüfung erfolgt in `SignedLicenseGate::check_ring` über eine strikte 7-stufige Fail-Closed-Sequenz (`src/signed_gate.rs:744-788`):
1. **Schritt 1 (Fast-Ring-Bypass):** `FeatureRing::Fast` benötigt keine Lizenz (Invariante `INV-LICENSE-2`, `src/signed_gate.rs:310-312`).
2. **Schritt 2 (Vorhandensein):** Prüfung auf Vorhandensein von `activation` oder `license_payload` (`src/signed_gate.rs:320-329`).
3. **Schritt 3 (Installationsbindung):** Prüfung von `installation_id_hash` in Constant-Time (`src/signed_gate.rs:341-370`).
4. **Schritt 4 (Signaturprüfung):** Prüfung der Ed25519-Signatur (`src/signed_gate.rs:380-398`).
5. **Schritt 5 (Ablaufprüfung):** Prüfung gegen `Clock`-Port (`src/signed_gate.rs:408-431`).
6. **Schritt 6 (Ring-Berechtigung):** Prüfung, ob der Ring erlaubt ist (`src/signed_gate.rs:441-457`).
7. **Schritt 7 (Erfolg):** Bestätigung (`src/signed_gate.rs:463-465`).

### 2.2 Verhalten bei Schlüsseldiskrepanz (Falscher / Unbekannter Schlüssel)
- Wird ein Token verifiziert, das mit Schlüssel A signiert wurde, während das Gate mit Schlüssel B instanziiert ist, schlägt Schritt 4 fehl:
  - `act.verify_signature(vk)` schlägt fehl (`src/signed_gate.rs:387-392`).
  - Interner Fehler: `StepVerificationError::InvalidSignature` (`src/signed_gate.rs:95, 391`).
  - Nach außen gemeldeter Fehler: `LicenseError::InvalidSignature` (`src/signed_gate.rs:120`).
- Wird ein Roh-Payload via `from_signed_payload*` mit dem falschen Schlüssel übergeben, schlägt die Verifikation bereits im Konstruktor fehl und liefert sofort `Err(LicenseError::InvalidSignature)` (`src/signed_gate.rs:586-589`).
- **Schutz vor Informationslecks:** Die Fehlermeldung lautet strikt `"activation signature verification failed"` (`crates/contextra-ports/src/license.rs:54`). Es werden keine Schlüsseldetails, Hash-Fragmente oder Schlüsselunterschiede nach außen exponiert (`tests/error_no_information_leak.rs:26-34`, `tests/key_rotation.rs`).

---

## 3. Betriebsanweisung für einen Schlüsselwechsel (Operational Procedure)

Da der Code keine unterbrechungsfreie Schlüsselrotation mit mehreren Schlüsseln unterstützt, muss ein Schlüsselwechsel im Betrieb als koordinierter Austauschprozess durchgeführt werden:

1. **Erzeugung des neuen Schlüsselpaars:**
   - Erzeuge einen neuen `SigningKey` und leite den zugehörigen `VerifyingKey` ab.
   - Sichere Aufbewahrung des privaten Schlüssels im KMS / HSM.
2. **Neuausstellung der Lizenz-Token:**
   - Signiere für jede Installation / jeden Mandanten ein neues Token (`SignedActivation` oder `LicensePayload`) mit dem neuen privaten Schlüssel.
3. **Rollout & Gate-Aktualisierung:**
   - Konfiguriere die Anwendung / den Dienst mit dem neuen `VerifyingKey` und dem neuen Token.
   - Da das Gate unveränderlich (immutable) mit genau einem Schlüssel arbeitet, erfolgt der Wechsel durch Instanziierung eines neuen `SignedLicenseGate`:
     ```rust
     let new_gate = SignedLicenseGate::from_activation(new_activation, new_verifying_key);
     // bzw. bei Payload-Bytes:
     let new_gate = SignedLicenseGate::from_signed_payload(&payload_bytes, &sig, new_verifying_key)?;
     ```
4. **Verifikation nach dem Wechsel:**
   - Stelle sicher, dass `new_gate.check_ring(ring)` erfolgreich `Ok(())` zurückgibt.
   - Verifiziere das Fail-Closed-Verhalten: Ein altes, mit dem vorherigen Schlüssel signiertes Token wird vom neuen Gate deterministisch mit `Err(LicenseError::InvalidSignature)` abgewiesen (`tests/key_rotation.rs:test_token_from_key_a_rejected_by_key_b`).

---

## 4. Bekannte Grenzen & Sicherheitsbefunde

1. **Kein Keyring / Multi-Key-Support:**
   - Keine Möglichkeit, zwei Schlüssel während einer Übergangsphase parallel als gültig zu führen.
   - Ein Schlüsselwechsel erfordert synchronen oder atomaren Austausch von Token und Verifikationsschlüssel im System.
2. **Fehlende Schlüsselkennung (`key_id`):**
   - Lizenzen enthalten keine Kennung des ausstellenden Schlüssels. Sollte künftig Multi-Key-Support implementiert werden, müssen entweder alle bekannten Schlüssel durchprobiert werden (Trial Decryption / Multi-Verify) oder das Lizenzformat muss um ein `key_id`-Feld erweitert werden.
3. **Fail-Closed bei Diskrepanz:**
   - Bei fehlerhafter Konfiguration oder Schlüsselkonflikt schaltet das System sofort in den Fail-Closed-Zustand (nur `FeatureRing::Fast` bleibt funktionsfähig, Enterprise-Ringe wie `Sovereign` oder `Compliance` werden gesperrt).
