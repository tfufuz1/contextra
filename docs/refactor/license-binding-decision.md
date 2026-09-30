# Architekturentscheidung: Lizenz-Binding & Activation Model (`LicensePayload` vs. `SignedActivation`)

**Status:** Vorschlag / Entscheidungsdokument
**Datum:** 2026-09-28
**Betroffene Crates:** `contextra-license`, `contextra-ports`, `contextra-engine`, `contextra`, `contextra-mcp`
**Referenzierte Spezifikationen:** Spec §14.6, §15, Spec-Anforderung D.1, ADR-104

---

## 1. Ausgangslage & Problemstellung

Die Spezifikation (Spec-Anforderung D.1) skizziert eine Aktivierungsstruktur zur Durchsetzung des Drei-Ring-Lizenzmodells:

```rust
struct SignedActivation {
    ring: FeatureRing,
    installation_id_hash: [u8; 32],
    expires_at_unix: i64,
    signature: [u8; 64],
}
```

Sowie den zugehörigen Evaluierungsalgorithmus für `check_ring(ring)`:
1. `FeatureRing::Fast` $\rightarrow$ `Ok(())` bedingungslos (auch bei fehlender oder korrupter Lizenz, Invariante `INV-LICENSE-2`).
2. Keine Aktivierung vorhanden $\rightarrow$ `Err(LicenseError::NotActivated(ring))`.
3. `installation_id_hash` $\neq$ lokale Hardware-/Instanz-Kennung $\rightarrow$ `Err(LicenseError::NotActivated(ring))` (*fail-closed ohne Informationsleck*).
4. Signatur ungültig $\rightarrow$ `Err(LicenseError::InvalidSignature)`.
5. `clock.now_unix()` $\ge$ `expires_at` $\rightarrow$ `Err(LicenseError::Expired(expires_at))`.
6. Aktivierter Ring niedriger als angefragt $\rightarrow$ `Err(LicenseError::NotActivated(ring))`.
7. `Ok(())`.

### IST-Zustand im Codebase

In der aktuellen Implementierung (`crates/contextra-license/src/signed_gate.rs`) existiert stattdessen der Typ `LicensePayload`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicensePayload {
    pub tenant_id: TenantId,
    pub allowed_rings: Vec<FeatureRing>,
    pub expires_at: Option<i64>,
    pub feature_flags: BTreeMap<String, bool>,
}
```

Die Validierung erfolgt in `SignedLicenseGate::from_signed_payload_with_clock(...)`, das Ed25519-Signaturen über den Bincode-serialisierten Payload prüft.

**Offene Entscheidung:** Soll die Datenstruktur auf `installation_id_hash` umgestellt werden, bei `tenant_id` verbleiben, oder um ein optionales/kombiniertes Binding erweitert werden?

---

## 2. Heutiger Stand: Typen, Fehlerarten und Aufrufer

### 2.1 Typen & Traits
* **`LicensePayload`** (`contextra-license`): Repräsentiert den signierten Inhalt (Tenant-ID, aktivierte Ringe, Ablaufsdatum, Feature-Flags).
* **`SignedLicenseGate`** (`contextra-license`): Implementiert das Trait `LicenseGate`. Hält den VerifyingKey (Ed25519) und den geprüften `LicensePayload`.
* **`LicenseGate` Trait** (`contextra-ports`):
  ```rust
  pub trait LicenseGate: Send + Sync {
      fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>;
      fn authorize(&self, requested: FeatureRing) -> Result<AuthorizedRing, LicenseError>;
  }
  ```
* **`LicenseError` Enum** (`contextra-ports`):
  ```rust
  pub enum LicenseError {
      NotActivated(FeatureRing),
      InvalidSignature,
      Expired(i64),
  }
  ```
* **`OpenFastGate`** (`contextra-ports`): Standard-Gate für Open-Source-Betrieb (erlaubt `Fast`, lehnt `Sovereign` / `Compliance` ab).
* **`AuthorizedRing`** (`contextra-ports`): Unfälschbares Token für geschützte Engine-Operationen (ADR-104).

### 2.2 Aufrufer von `check_ring` bzw. `LicenseGate`
1. **`contextra` Facade (`PerformanceProfile::enforce_license`)**: Prüft beim Bauen von Collections/Engines, ob das gewählte Performance-Profil (`Fast`, `Sovereign`, `Compliance`) durch die Lizenz abgedeckt ist.
2. **`contextra::builder::EngineBuilder`**: Nimmt `Arc<dyn LicenseGate>` oder Payload-Bytes + Signatur entgegen.
3. **`contextra-engine::Engine`**: Speichert `license_gate: Arc<dyn LicenseGate>` zur Laufzeitprüfung.
4. **`contextra-ports::PluginRegistry`**: Prüft bei der Registrierung von Plugins, ob der erforderliche Ring freigeschaltet ist (`gate.check_ring(required_ring)`).
5. **`contextra-mcp`**: Verwendet die `contextra`-Engine als Backend und stützt sich auf das dort konfigurierte `LicenseGate`.

---

## 3. Analyse der Optionen

### Option A: Nur `tenant_id` (Status Quo)
Die Lizenz bindet ausschließlich an eine logische Organisation bzw. Mandanten-Identität (`TenantId`).

* **Vorteile:**
  * Maximale Flexibilität für Cloud-Native, Kubernetes Pods, Auto-Scaling und ephemere Container.
  * Kein lokales Machine-ID-Fingerprinting erforderlich (kein Lesezugriff auf `/etc/machine-id`, MAC-Adressen oder DMI/Bios-Werte).
  * Einfaches Deployment für MCP-Server (`contextra-mcp`) und CLI-Tools.
* **Nachteile:**
  * Ein vergebener Lizenzschlüssel kann auf einer beliebigen Anzahl physischer oder virtueller Rechner kopiert und ausgeführt werden.

---

### Option B: Erweiterung um `installation_id_hash` (Hybrides Modell)
`LicensePayload` wird um ein optionales Feld `installation_id_hash: Option<[u8; 32]>` erweitert:

```rust
pub struct LicensePayload {
    pub tenant_id: TenantId,
    pub installation_id_hash: Option<[u8; 32]>,
    pub allowed_rings: Vec<FeatureRing>,
    pub expires_at: Option<i64>,
    pub feature_flags: BTreeMap<String, bool>,
}
```

* **Vorteile:**
  * Vereint organisatorische Zuordnung (`tenant_id`) mit optionalem Hardware/Node-Pinning (`installation_id_hash`).
  * B2B-Enterprise-Kunden mit On-Premises Appliances können an spezifische Server-Hardware gebunden werden.
  * Cloud-/MCP-Nutzer können lizenziert werden, indem `installation_id_hash = None` gesetzt wird.
* **Nachteile:**
  * Geringfügige Vergrößerung der Payload-Struktur.
  * Erfordert eine definierte Strategie zur Ermittlung der lokalen `installation_id` auf der Host-Maschine (sofern gesetzt).

---

### Option C: `tenant_id` durch `installation_id_hash` ersetzen (Exakte Spec D.1)
Entfernung von `tenant_id` aus der Aktivierungsstruktur zugunsten von `installation_id_hash: [u8; 32]`.

* **Vorteile:**
  * Exakte 1:1 Abdeckung des Datenstrukturentwurfs aus Spec-Anforderung D.1.
  * Verhindert das Kopieren der Lizenzdatei auf andere Rechner.
* **Nachteile:**
  * Verlust der direkten Mandanten-Verknüpfung (`TenantId`) im Lizenz-Header (relevante Zuordnung für Audit-Export und Multi-Tenant Scopes geht verloren).
  * Inkompatibel mit dynamischen Cloud-Umgebungen und containerisiertem `contextra-mcp`, da bei jedem Container-Neustart/Migration eine neue Lizenz ausgestellt werden müsste.

---

## 4. Auswirkungen auf Ring-Modell, `FeatureRing` und `contextra-mcp`

1. **Drei-Ring-Modell (`Fast`, `Sovereign`, `Compliance`):**
   * **`Fast` (MIT/Apache-2.0):** Durch die Invariante `INV-LICENSE-2` muss `check_ring(FeatureRing::Fast)` stets `Ok(())` zurückgeben — völlig unabhängig davon, ob `installation_id_hash` oder `tenant_id` übereinstimmen oder ob die Lizenz abgelaufen/korrupt ist.
   * **`Sovereign` / `Compliance`:** Diese kommerziellen bzw. funktional geschützten Ringe verlangen ein gültiges `SignedLicenseGate`.

2. **Impact auf `contextra-mcp`:**
   * MCP-Server laufen lokal auf Entwickler-Laptops oder in kurzlebigen Sidecar-Containern.
   * Eine strikte Pflicht-Bindung an `installation_id_hash` (Option C) würde die Nutzbarkeit von `contextra-mcp` im Enterprise-Umfeld massiv erschweren, da Hardware-Wechsel oder Container-Rebuilds Lizenzen ungültig machen würden. Option A oder Option B (mit `None` für MCP) bieten hier nahtlose Integration.

3. **Risiko für bestehende Signaturen und Formate:**
   * Jede Anpassung von `LicensePayload` ändert die Bincode-Serialisierung. Da Ed25519 über das exakte Byte-Array signiert, führt jede Schema-Änderung zur Entwertung aller zuvor ausgetauschten Test- und Produktions-Signaturen.
   * Bincode ist im Gegensatz zu JSON nicht feld-tolerant. Schema-Migrationen müssen daher strikt versioniert oder in einem Schritt durchgeführt werden.

---

## 5. Fundamentale Grenze: Der Open-Source / Fork-Bypass

Ein zentraler Aspekt für die Architekturentscheidung ist die quelloffene Natur von Contextra:

> **Fundamentale Grenze:**
> Die Kernbibliothek (Ring 0–2 und Facade) steht unter den Open-Source-Lizenzen MIT bzw. Apache-2.0.
> Ein technisches Hardware-Binding (`installation_id_hash`) innerhalb des Open-Source-Codes stellt **keinen unüberwindbaren Kopierschutz (DRM)** dar. Jeder Nutzer mit Rust-Kenntnissen kann das Repository forken und in `SignedLicenseGate::check_ring` das Matching von `installation_id_hash` oder die Signaturprüfung schlicht mit `Ok(())` überschreiben.

### Konsequenz für die Geschäfts- und Sicherheitsarchitektur
* Die Lizenzprüfung in `contextra-license` dient der **Legal Compliance und Audit-Sicherheit** für B2B-Kunden (z. B. Nachweis gegenüber Datenschutzbeauftragten, BSI-Grundschutz, DSGVO Artikel 30 Registrierung).
* B2B-Unternehmen nutzen die signierte Lizenz zur Vermeidung von Haftungsrisiken. Sie forken den Code nicht, um Lizenzschlüssel zu fälschen.
* Ein hochkomplexes Hardware-Fingerprinting (`installation_id_hash`) erzeugt signifikanten Wartungsaufwand (z. B. Supportfälle bei CPU-/MAC-Wechseln oder VM-Migrationen), ohne echten Schutz gegen böswillige Akteure zu bieten.

---

## 6. Empfehlung & Begründung

### Beschlussempfehlung: Option B (Hybrides Modell mit optionalem `installation_id_hash`)

Es wird empfohlen, `LicensePayload` wie folgt anzupassen:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicensePayload {
    /// Mandanten-Identität für Audit-Trail und Multi-Tenancy-Zuordnung.
    pub tenant_id: TenantId,
    /// Optionaler SHA-256 Hash der Instanz-/Hardware-Kennung für On-Premises Hardware-Binding.
    pub installation_id_hash: Option<[u8; 32]>,
    /// Freigeschaltete Feature-Ringe.
    pub allowed_rings: Vec<FeatureRing>,
    /// Optionaler Ablauf-Zeitstempel (Unix-Epoch in Sekunden).
    pub expires_at: Option<i64>,
    /// Zusätzliche feingranulare Feature-Flags.
    pub feature_flags: BTreeMap<String, bool>,
}
```

### Begründung
1. **Erfüllung der Spec D.1 ohne Nachteile:** Der in Spec D.1 geforderte `installation_id_hash` wird unterstützt, ohne die logische `TenantId` aufzugeben.
2. **Praxistauglichkeit für Cloud & MCP:** Wenn `installation_id_hash` `None` ist, funktioniert die Lizenz reibungslos in Kubernetes, Docker und `contextra-mcp`.
3. **Hardware-Binding bei Bedarf:** Enterprise-Kunden mit Appliance-Verträgen können ein fixes Hash-Binding nutzen (`Some([u8; 32])`).
4. **Verhältnismäßigkeit:** Berücksichtigt die Realität von MIT/Apache-2.0 Software: Legal Compliance steht im Vordergrund, Frustration für ehrliche Nutzer durch fehlerhaftes Hardware-Fingerprinting wird vermieden.
