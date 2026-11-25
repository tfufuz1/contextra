# Architektur-Entscheidung: Lizenz- & Aktivierungs-Binding (Tenant-ID vs. Installation-Binding)

**Datum:** 2026-09-28
**Status:** Entscheidungsentwurf (Phase 1)
**Betroffene Crates:** `contextra-license`, `contextra-ports`, `contextra-mcp`, `contextra`

---

## 1. Heutiger Stand (IST-Zustand)

### 1.1 Typen & Datenstrukturen
In `crates/contextra-license/src/signed_gate.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicensePayload {
    /// Tenant identity bound to this license.
    pub tenant_id: TenantId,
    /// List of feature rings activated by this license.
    pub allowed_rings: Vec<FeatureRing>,
    /// Optional Unix timestamp (in seconds) after which the license expires.
    pub expires_at: Option<i64>,
    /// Arbitrary key-value feature flags.
    pub feature_flags: BTreeMap<String, bool>,
}
```

* `SignedLicenseGate` speichert eine `verifying_key: VerifyingKey`, eine deserialisierte `license_payload: LicensePayload` sowie eine `clock: Arc<dyn Clock>`.
* Die Konstruktion (`from_signed_payload`) prüft die Ed25519-Signatur über den Bincode-serialisierten `LicensePayload`.
* In `check_ring(ring)` prüft das Gate:
  1. Ist `ring == FeatureRing::Fast`? Falls ja: `Ok(())` (`INV-LICENSE-2`).
  2. Ist `ring` in `allowed_rings` enthalten? Falls nein: `Err(LicenseError::NotActivated(ring))`.
  3. Ist `expires_at` gesetzt und `now_secs >= expires_at`? Falls ja: `Err(LicenseError::Expired(expires_at))`.

In `crates/contextra-ports/src/license.rs`:
```rust
pub enum LicenseError {
    NotActivated(FeatureRing),
    InvalidSignature,
    Expired(i64),
}

pub trait LicenseGate: Send + Sync {
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>;
    fn authorize(&self, requested: FeatureRing) -> Result<AuthorizedRing, LicenseError>;
}
```

### 1.2 Aufrufer von `check_ring` bzw. `LicenseGate`
1. **`contextra` Facade (`builder.rs`, `performance_profile.rs`)**:
   `ContextraBuilder` erzwingt die Autorisierung des gewählten `FeatureRing` bei der Instanziierung der Engine.
2. **`contextra-ports` (`plugin.rs`)**:
   `PluginRegistry::activate_all()` prüft vor der Aktivierung von Plugins, ob der geforderte `FeatureRing` freigeschaltet ist (`LicenseError::NotActivated`).
3. **`contextra-mcp` (`plugin_status.rs`)**:
   Verwendet `FeatureRing`, um den aktiven Lizenzstatus gegenüber MCP-Clients zu exponieren.

---

## 2. Spezifikations-Anforderung (Spec D.1)

Laut Spezifikationsanforderung **D.1** wird ein kompaktes, direktes Aktivierungsmodell definiert:

### 2.1 Datenstruktur (`SignedActivation`)
* Fields: `ring: FeatureRing`, `installation_id_hash: [u8; 32]`, `expires_at_unix: i64`, `signature: [u8; 64]`.
* Signatur: Ed25519-Signatur direkt über die drei Datenfelder (`ring`, `installation_id_hash`, `expires_at_unix`).

### 2.2 Prüflogik in `check_ring(requested_ring)`
1. **Fast-Ring**: `requested_ring == Fast` $\rightarrow$ `Ok(())` immer (auch bei korrupter/fehlender Aktivierung, `INV-LICENSE-2`).
2. **Keine Aktivierung vorhanden**: $\rightarrow$ `Err(LicenseError::NotActivated)`.
3. **Mismatch der Kennung**: `installation_id_hash != lokale_installation_id_hash` $\rightarrow$ `Err(LicenseError::NotActivated)` (Sicherheits-Grundsatz: Kein Informationsleck bezüglich der erwarteten/tatsächlichen ID).
4. **Ungültige Signatur**: $\rightarrow$ `Err(LicenseError::InvalidSignature)`.
5. **Ablauf**: `clock.now_unix() >= expires_at` $\rightarrow$ `Err(LicenseError::Expired)`.
6. **Ring-Stufe unzureichend**: Aktivierter Ring niedriger als angefragt $\rightarrow$ `Err(LicenseError::NotActivated)`.
7. **Erfolg**: $\rightarrow$ `Ok(())`.

---

## 3. Handlungsoptionen

| Kriterium | Option A: Nur `tenant_id` (IST) | Option B: Hybrid (`tenant_id` + `installation_id_hash`) | Option C: Ersetzung gemäß Spec D.1 (`SignedActivation`) |
|---|---|---|---|
| **Datenstruktur** | `LicensePayload` (Tenant, Rings, Expire, Flags) | `LicensePayload` erweitert um `installation_id_hash: Option<[u8; 32]>` | `SignedActivation` (`ring`, `installation_id_hash`, `expires_at`) |
| **Spec D.1 Konformität** | Abweichend | Großteils konform (Superset) | 100 % konform |
| **On-Premise Instanz-Bindung** | Nein (Lizenz gilt pro Tenant) | Ja (optional oder erzwungen) | Ja (strikt) |
| **Mandanten-Bezug (`TenantId`)** | Explizit in Lizenz enthalten | Explizit in Lizenz enthalten | Nicht in Lizenz (wird durch Anwendung verwaltet) |
| **Feature Flags** | Flexibel in Lizenz integriert | Flexibel in Lizenz integriert | Entfällt in Lizenz (in App/Config verwaltet) |

### Option A: Beibehalten von `tenant_id` (Keine Installation-Bindung)
* **Vorteile:** Kein Refactoring erforderlich; Mandanten-Bindung direkt im Lizenzobjekt.
* **Nachteile:** Weicht von Spec D.1 ab; Lizenzen können ohne Weiteres auf beliebigen Servern kopiert und wiederverwendet werden, solange die `TenantId` übereinstimmt.

### Option B: Hybrid-Erweiterung (`tenant_id` + `installation_id_hash`)
* **Vorteile:** Kombiniert Mandanten-Kontext mit der hardware-/instanzbezogenen Diebstahlssicherung; abwärtskompatibel gestaltbar.
* **Nachteile:** Payload-Format ist komplexer als in Spec D.1 vorgesehen; Bincode-Deserialisierung muss Feld-Erweiterung abfangen.

### Option C: Vollständige Ersetzung durch `SignedActivation` (Spec D.1)
* **Vorteile:** Exakte Einhaltung der Spec D.1; Schlankes binäres Format ohne Bincode/Map-Overhead; strikter Instanzschutz.
* **Nachteile:** Bisherige `LicensePayload`-Signaturen werden ungültig; `feature_flags` und `tenant_id` müssen außerhalb der Lizenz-Aktivierung verarbeitet werden.

---

## 4. Auswirkungen auf Architektur, Ringe und MCP

### 4.1 Ring-Modell & `FeatureRing`
* Das dreistufige Ring-Modell (`Fast`, `Sovereign`, `Compliance`) bleibt in allen Optionen unverändert.
* Invariante `INV-LICENSE-2` (Fail-Open für `Fast`) bleibt bei allen Optionen garantiert.

### 4.2 `contextra-mcp` & Plugin-System
* `contextra-mcp` fragt über `plugin_status.rs` den aktiven `FeatureRing` ab.
* Bei Option B & C muss beim Start des MCP-Servers oder der Engine die lokale `installation_id_hash` (z. B. BLAKE3-Hash aus Machine-ID, MAC-Adresse oder Instanz-UUID) bestimmt und an das `SignedLicenseGate` übergeben werden.

### 4.3 Risiko für bestehende Signaturen und Formate
* Da `contextra-license` als `experimental` eingestuft ist und das Lizenzformat bisher nur in internen Modultests genutzt wird, ist das Risiko eines Breaking Changes bei Option B oder C minimal.

---

## 5. Systemische Grenze (Open-Source-Grenzziehung)

**Wichtige architektonische Erkenntnis:**
Contextra wird unter einer quelloffenen Lizenz (MIT / Apache-2.0) als Rust-Bibliothek ausgeliefert.

* **Grenze:** Da der Quellcode frei zugänglich ist und im Prozess des Anwenders läuft, kann jede clientseitige Lizenz- und Aktivierungsprüfung (einschließlich `check_ring` und Ed25519-Signaturprüfungen) durch ein Einkompilieren einer eigenen `LicenseGate`-Implementierung oder einen einfachen Fork umgangen werden.
* **Zweck des Aktivierungssystems:** Das Lizenz-System und das Installation-Binding dienen **nicht** als unknackbares DRM, sondern als **rechtlich und audittechnisch relevantes Compliance-Gate**. Es schützt gewerbliche Nutzer vor unabsichtlicher Fehllizenzierung ("License Drift") und bildet die Grundlage für kommerzielle Support- und BSI-/DSGVO-Compliance-Verträge im `Compliance`-Ring.

---

## 6. Empfehlung mit Begründung

**Empfehlung: Umsetzung von Option B (Hybrid) oder Option C (Vollständige Spec-Konformität)**

1. **Falls die Spezifikation D.1 strikt bindend ist:**
   Implementierung von **Option C (`SignedActivation`)**. `LicensePayload` wird durch `SignedActivation` ersetzt. Die Prüfung von `installation_id_hash` erfolgt gegen einen beim Erstellen des Gates injizierten lokalen Hash.
2. **Falls Mandanten-Metadaten in der Lizenz benötigt werden:**
   Implementierung von **Option B**. `LicensePayload` wird um `pub installation_id_hash: Option<[u8; 32]>` erweitert. Wenn das Feld gesetzt ist, prüft `check_ring` die Übereinstimmung mit der lokalen Installations-ID.

*Entscheidungsvorschlag für das Team:* **Option C** umsetzen, um 100 % Konformität mit Spec D.1 herzustellen, da Mandanten-Zuordnungen auf Engine-Ebene (`TenantId`) unabhängig von der Instanz-Aktivierung verwaltet werden.
