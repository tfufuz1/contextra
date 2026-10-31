//! Kryptografisch signierte Lizenz-Gate-Implementierung (§10.2, §14.6, §15).
//!
//! Dieses Modul implementiert die Offline-Verifikation von Lizenzen und Feature-Ringen.
//! Es ermöglicht Contextra-Knoten, erweiterte Funktionen (z. B. Sovereign- oder Compliance-Ring)
//! ohne bestehende Internet- oder Cloud-Verbindung abzusichern.
//!
//! # Architektur & Schutzziele
//! - **Offline-Verifikation:** Prüfung ausschließlich über asymmetrische Ed25519-Signaturen
//!   und kryptografische BLAKE3-Hashes (REQ-LIC-001). Keine externen Netzwerkaufrufe.
//! - **Hardware- & Installations-Bindung:** Lizenzen können an eine konkrete Maschine oder
//!   Installation gebunden werden (`installation_id_hash`). Ein Kopieren auf fremde Hosts schlägt fehl.
//! - **Deterministische Zeitprüfung:** Zeitabfragen erfolgen strikt über den [`Clock`]-Port
//!   (Regel 3: Ports für Zeit, Zufall und IDs). Dies erlaubt deterministische Tests sowie Schutz vor Replay.
//! - **Fail-Closed Prinzip:** Tritt ein Fehler, eine fehlende Aktivierung oder ein ungültiger Hash auf,
//!   wird der Zugriff verweigert (`Err(LicenseError::...)`). Nur der freie `Fast`-Ring ist immer freigeschaltet (INV-LICENSE-2).

use std::collections::BTreeMap;
use std::sync::Arc;

use bincode::Options;
use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_types::TenantId;
pub use ed25519_dalek::VerifyingKey;
use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ANCHOR[contextra-license:CONST-MAX-PAYLOAD]
/// Maximale zulässige Payload-Größe in Bytes (64 KiB) für die Lizenzverifikation.
///
/// **Zweck:** Schutz vor Speicher- und CPU-Erschöpfungsangriffen (Denial of Service)
/// durch übergroße oder manipulierte Nutzlasten bei der Deserialisierung.
pub const MAX_PAYLOAD_BYTES: usize = 65_536;

// ANCHOR[contextra-license:FN-CONSTANT-TIME-EQ]
/// Vergleicht zwei 32-Byte-Arrays in konstanter Laufzeit (Constant-Time).
///
/// **Zweck:** Verhindert Seitenkanal-Angriffe (Timing Attacks) beim Prüfen von kryptografischen
/// Hashes oder geheimen Schlüsseln, indem alle Bytes unabhängig von früheren Diskrepanzen
/// bitweise verknüpft werden.
///
/// **Ablauf:**
/// 1. Iteriert über alle 32 Bytes beider Slices synchron.
/// 2. Bitweises XOR (`^`) akkumuliert Unterschiede in einer Hilfsvariablen via OR (`|=`).
/// 3. Gibt `true` genau dann zurück, wenn das Gesamtergebnis 0 ist (alle Bytes identisch).
///
/// **Nebenwirkungen:** Keine (reine CPU-Operation im Stack).
#[inline]
pub fn constant_time_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut res = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        res |= x ^ y;
    }
    res == 0
}

// ANCHOR[contextra-license:STRUCT-OPTIONS]
/// Konfigurationsoptionen für die Prüfung signierter Payloads und Rückwärtskompatibilität.
///
/// Ermöglicht es, die maximale Nutzlastgröße anzupassen oder alte Payload-Schemata
/// ohne Installations-ID-Hash explizit zuzulassen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedLicenseOptions {
    /// Gibt an, ob der Fallback auf das Legacy-Schema (ohne Installations-Hash) erlaubt ist.
    pub allow_legacy: bool,
    /// Obergrenze für die Größe des serialisierten Payloads in Bytes.
    pub max_payload_bytes: usize,
}

impl Default for SignedLicenseOptions {
    /// Standardoptionen: Strikter Modus (kein Legacy erlaubt) und 64 KiB Obergrenze.
    fn default() -> Self {
        Self {
            allow_legacy: false,
            max_payload_bytes: MAX_PAYLOAD_BYTES,
        }
    }
}

// ANCHOR[contextra-license:ENUM-STEP-ERROR]
/// Detaillierte Fehlerursache während der 7-stufigen Lizenz- und Aktivierungsprüfung.
///
/// **Zweck:** Ermöglicht intern eine präzise Diagnose, welcher Schritt fehlgeschlagen ist,
/// wird jedoch über [`StepVerificationError::into_license_error`] in standardisierte,
/// informationsleck-freie [`LicenseError`]-Varianten überführt.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StepVerificationError {
    /// Schritt 2: Weder ein Aktivierungsdatensatz noch ein Lizenz-Payload ist vorhanden.
    #[error("activation or license payload missing for ring {0:?}")]
    MissingActivation(FeatureRing),
    /// Schritt 3: Der Hash der lokalen Installation stimmt nicht mit der Lizenz überein.
    #[error("installation ID hash mismatch for ring {0:?}")]
    InstallationMismatch(FeatureRing),
    /// Schritt 4: Die kryptografische Ed25519-Signatur ist ungültig oder der öffentliche Schlüssel fehlt.
    #[error("activation signature verification failed")]
    InvalidSignature,
    /// Schritt 5: Die Lizenz ist zeitlich abgelaufen (`current_time >= expires_at`).
    #[error("activation expired at {expires_at} (current time: {current_time})")]
    Expired {
        /// Unix-Zeitstempel (in Sekunden), an dem die Lizenz ablief.
        expires_at: i64,
        /// Aktuelle System- bzw. Port-Zeit in Sekunden.
        current_time: i64,
    },
    /// Schritt 6: Der angeforderte Feature-Ring ist in der Lizenz nicht freigeschaltet.
    #[error("ring {0:?} is not permitted by activation")]
    RingNotPermitted(FeatureRing),
}

impl StepVerificationError {
    // ANCHOR[contextra-license:FN-STEP-ERROR-CONV]
    /// Überführt den internen Schrittfehler in den kanonischen [`LicenseError`].
    ///
    /// **Garantie gegen Informationslecks:** Hardware-Hashes, Maschinendaten oder genaue
    /// Abweichungen werden nicht nach außen propagiert, um Angreifern keine Hinweise zu liefern.
    pub fn into_license_error(self) -> LicenseError {
        match self {
            Self::MissingActivation(ring) => LicenseError::NotActivated(ring),
            Self::InstallationMismatch(ring) => LicenseError::NotActivated(ring),
            Self::InvalidSignature => LicenseError::InvalidSignature,
            Self::Expired { expires_at, .. } => LicenseError::Expired(expires_at),
            Self::RingNotPermitted(ring) => LicenseError::NotActivated(ring),
        }
    }
}

// ANCHOR[contextra-license:MOD-BYTES64]
/// Hilfsmodul für die Serde-Serialisierung und -Deserialisierung von 64-Byte-Arrays (Signaturen).
mod bytes_64 {
    use super::*;

    /// Serialisiert ein 64-Byte-Array als rohe Bytefolge.
    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    /// Deserialisiert eine Bytefolge und prüft strikt auf exakt 64 Bytes Länge.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = serde::Deserialize::deserialize(deserializer)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes for signature"))
    }
}

// ANCHOR[contextra-license:STRUCT-SIGNED-ACTIVATION]
/// Signierter Aktivierungsdatensatz zur Bindung eines Feature-Rings an eine Maschinenidentität (§10.2, §15).
///
/// Verbindet kryptografisch den freizuschaltenden Feature-Ring, einen 32-Byte-BLAKE3-Hash
/// der Zielinstallation und ein Ablaufdatum mit einer Ed25519-Signatur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedActivation {
    /// Der freizuschaltende Feature-Ring (z. B. `FeatureRing::Sovereign`).
    pub ring: FeatureRing,
    /// 32-Byte-BLAKE3-Hash der Ziel-Installationskennung.
    pub installation_id_hash: [u8; 32],
    /// Ablaufdatum als Unix-Zeitstempel in Sekunden.
    pub expires_at_unix: i64,
    /// 64-Byte-Ed25519-Signatur über das Tupel `(ring, installation_id_hash, expires_at_unix)`.
    #[serde(with = "bytes_64")]
    pub signature: [u8; 64],
}

impl SignedActivation {
    // ANCHOR[contextra-license:FN-ACTIVATION-PAYLOAD-BYTES]
    /// Berechnet die binäre Repräsentation der Aktivierungsfelder zur Signaturerstellung/-prüfung.
    ///
    /// **Ablauf:** Serialisiert das Tripel `(ring, installation_id_hash, expires_at_unix)` mittels Bincode.
    /// Schlägt die Serialisierung fehl, wird ein leerer Vektor zurückgegeben.
    pub fn payload_bytes(
        ring: FeatureRing,
        installation_id_hash: [u8; 32],
        expires_at_unix: i64,
    ) -> Vec<u8> {
        bincode::serialize(&(ring, installation_id_hash, expires_at_unix)).unwrap_or_default()
    }

    // ANCHOR[contextra-license:FN-ACTIVATION-CREATE-SIGNED]
    /// Erstellt und signiert einen neuen Aktivierungsdatensatz mit dem privaten Ed25519-Schlüssel.
    ///
    /// **Ablauf:**
    /// 1. Berechnet die Bytefolge via [`Self::payload_bytes`].
    /// 2. Signiert die Bytes mit dem übergebenen `signing_key`.
    /// 3. Gibt die vollständige Instanz von [`SignedActivation`] zurück.
    pub fn create_signed(
        ring: FeatureRing,
        installation_id_hash: [u8; 32],
        expires_at_unix: i64,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Self {
        let payload = Self::payload_bytes(ring, installation_id_hash, expires_at_unix);
        let signature = signing_key.sign(&payload).to_bytes();
        Self {
            ring,
            installation_id_hash,
            expires_at_unix,
            signature,
        }
    }

    // ANCHOR[contextra-license:FN-ACTIVATION-VERIFY-SIG]
    /// Verifiziert die Ed25519-Signatur dieses Datensatzes anhand des öffentlichen Schlüssels.
    ///
    /// **Rückgabe:** `true` genau dann, wenn die Signatur gültig ist und den Daten entspricht.
    pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> bool {
        let payload =
            Self::payload_bytes(self.ring, self.installation_id_hash, self.expires_at_unix);
        let sig = Signature::from_bytes(&self.signature);
        verifying_key.verify(&payload, &sig).is_ok()
    }
}

// ANCHOR[contextra-license:FN-DERIVE-INSTALLATION-ID]
/// Ermittelt oder erzeugt deterministisch den 32-Byte-BLAKE3-Hash der lokalen Installation.
///
/// # Status
/// **VORLÄUFIG, BIS ZUR SCHRIFTLICHEN ENTSCHEIDUNG** (Spec Teil 15, „Offene Entscheidungen“).
///
/// **Ablauf:**
/// 1. Falls ein Speicherpfad (`storage_path`) übergeben wird:
///    - Versucht, eine Datei namens `installation.id` darin zu lesen. Ist der Inhalt nicht leer,
///      wird dessen BLAKE3-Hash zurückgegeben.
///    - Existiert die Datei nicht, wird eine neue ID (`inst-<hex-hash-des-pfads>`) generiert,
///      in die Datei geschrieben und ihr Hash zurückgegeben.
/// 2. Falls kein Pfad angegeben ist:
///    - Nutzt einen festen Fallback-Seed (`b"CONTEXTRA_DEFAULT_LOCAL_INSTALLATION_ID_v1"`),
///      um einen deterministischen Hash abzuleiten.
///
/// **Nebenwirkungen:** Dateisystem-I/O (Lesen/Schreiben), falls ein Pfad angegeben ist.
pub fn derive_local_installation_id_hash(storage_path: Option<&std::path::Path>) -> [u8; 32] {
    if let Some(path) = storage_path {
        let id_file = path.join("installation.id");
        if let Ok(content) = std::fs::read_to_string(&id_file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return *blake3::hash(trimmed.as_bytes()).as_bytes();
            }
        }
        let generated = format!(
            "inst-{}",
            blake3::hash(path.to_string_lossy().as_bytes()).to_hex()
        );
        if std::fs::create_dir_all(path).is_ok() {
            if let Ok(()) = std::fs::write(&id_file, &generated) {}
        }
        return *blake3::hash(generated.as_bytes()).as_bytes();
    }

    const FALLBACK_SEED: &[u8] = b"CONTEXTRA_DEFAULT_LOCAL_INSTALLATION_ID_v1";
    *blake3::hash(FALLBACK_SEED).as_bytes()
}

// ANCHOR[contextra-license:STRUCT-LICENSE-PAYLOAD]
/// Strukturierter Lizenznutzlast-Datensatz (Payload).
///
/// Enthält Mandantenbindung ([`TenantId`]), freigeschaltete Feature-Ringe, optionales Ablaufdatum,
/// benutzerdefinierte Feature-Flags und einen optionalen Installations-ID-Hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicensePayload {
    /// Mandantenidentität, an die diese Lizenz gebunden ist.
    pub tenant_id: TenantId,
    /// Liste aller durch diese Lizenz freigeschalteten Feature-Ringe.
    pub allowed_rings: Vec<FeatureRing>,
    /// Optionaler Unix-Zeitstempel (in Sekunden), an dem die Lizenz abläuft (`None` = unbegrenzt).
    pub expires_at: Option<i64>,
    /// Zusätzliche Feature-Flags als Schlüssel-Wert-Paare.
    pub feature_flags: BTreeMap<String, bool>,
    /// Optionaler 32-Byte-Hash der lokalen Installations- oder Maschinenkennung.
    pub installation_id_hash: Option<[u8; 32]>,
}

// ANCHOR[contextra-license:STRUCT-LEGACY-PAYLOAD]
/// Altes Lizenz-Payload-Schema ohne `installation_id_hash` zur Rückwärtskompatibilität.
#[derive(Deserialize)]
struct LegacyLicensePayload {
    tenant_id: TenantId,
    allowed_rings: Vec<FeatureRing>,
    expires_at: Option<i64>,
    feature_flags: BTreeMap<String, bool>,
}

impl From<LegacyLicensePayload> for LicensePayload {
    /// Konvertiert ein Legacy-Payload in das moderne Schema mit `installation_id_hash: None`.
    fn from(legacy: LegacyLicensePayload) -> Self {
        Self {
            tenant_id: legacy.tenant_id,
            allowed_rings: legacy.allowed_rings,
            expires_at: legacy.expires_at,
            feature_flags: legacy.feature_flags,
            installation_id_hash: None,
        }
    }
}

// ============================================================================
// Sieben Verifikationsschritte (Fail-Closed Sequenz)
// ============================================================================

// ANCHOR[contextra-license:STEP-1-FAST-RING]
/// Schritt 1: Prüfung auf Fast-Ring-Bypass (Invariante INV-LICENSE-2).
///
/// **Garantie:** Der freie Standard-Ring `FeatureRing::Fast` benötigt niemals eine Lizenz
/// und darf nicht blockiert werden. Gibt `true` zurück, wenn alle weiteren Prüfungen übersprungen werden können.
pub fn verify_step_1_fast_ring(ring: FeatureRing) -> bool {
    ring == FeatureRing::Fast
}

// ANCHOR[contextra-license:STEP-2-PRESENCE]
/// Schritt 2: Vorhandenseinsprüfung von Aktivierungsdatensatz oder Lizenzpayload.
///
/// **Vorbedingung:** Schritt 1 war `false` (Ring ist nicht `Fast`).
/// **Fehler:** Gibt [`StepVerificationError::MissingActivation`] zurück, falls weder
/// ein [`SignedActivation`] noch ein [`LicensePayload`] übergeben wurde.
pub fn verify_step_2_presence(
    activation: Option<&SignedActivation>,
    payload: Option<&LicensePayload>,
    ring: FeatureRing,
) -> Result<(), StepVerificationError> {
    if activation.is_none() && payload.is_none() {
        return Err(StepVerificationError::MissingActivation(ring));
    }
    Ok(())
}

// ANCHOR[contextra-license:STEP-3-INSTALLATION]
/// Schritt 3: Prüfung auf Übereinstimmung der lokalen Installations-ID.
///
/// Vergleicht den Hash der aktuellen Maschinenumgebung in konstanter Zeit mit dem in der Lizenz
/// hinterlegten Hash ([`constant_time_eq_32`]).
///
/// **Fehler:**
/// - Gibt [`StepVerificationError::InstallationMismatch`] zurück, wenn die Hashes nicht übereinstimmen
///   oder die lokale ID fehlt.
/// - Bei Legacy-Payloads ohne `installation_id_hash` wird eine Warnung protokolliert und die Prüfung toleriert.
pub fn verify_step_3_installation(
    activation: Option<&SignedActivation>,
    payload: Option<&LicensePayload>,
    local_installation_id: Option<&[u8; 32]>,
    ring: FeatureRing,
) -> Result<(), StepVerificationError> {
    if let Some(act) = activation {
        match local_installation_id {
            Some(local_hash) if constant_time_eq_32(local_hash, &act.installation_id_hash) => {
                Ok(())
            }
            _ => Err(StepVerificationError::InstallationMismatch(ring)),
        }
    } else if let Some(pld) = payload {
        if let Some(ref expected_hash) = pld.installation_id_hash {
            match local_installation_id {
                Some(local_hash) if constant_time_eq_32(local_hash, expected_hash) => Ok(()),
                _ => Err(StepVerificationError::InstallationMismatch(ring)),
            }
        } else {
            // NOTE: Legacy-Lizenzen enthalten keine Hardware-Bindung; zur Rückwärtskompatibilität überspringen.
            tracing::warn!(
                "LicensePayload has no installation_id_hash (legacy payload); skipping installation ID check"
            );
            Ok(())
        }
    } else {
        Err(StepVerificationError::MissingActivation(ring))
    }
}

// ANCHOR[contextra-license:STEP-4-SIGNATURE]
/// Schritt 4: Kryptografische Ed25519-Signaturprüfung.
///
/// Prüft, ob der Aktivierungsdatensatz mit dem angegebenen öffentlichen Schlüssel (`verifying_key`)
/// gültig signiert wurde.
///
/// **Fehler:** Gibt [`StepVerificationError::InvalidSignature`] zurück, falls kein Schlüssel
/// vorhanden ist oder die Signaturprüfung fehlschlägt.
pub fn verify_step_4_signature(
    activation: Option<&SignedActivation>,
    payload: Option<&LicensePayload>,
    verifying_key: Option<&VerifyingKey>,
) -> Result<(), StepVerificationError> {
    if let Some(act) = activation {
        let sig_valid = match verifying_key {
            Some(vk) => act.verify_signature(vk),
            None => false,
        };
        if !sig_valid {
            return Err(StepVerificationError::InvalidSignature);
        }
    } else if payload.is_some() && verifying_key.is_none() {
        return Err(StepVerificationError::InvalidSignature);
    }
    // NOTE: Für LicensePayload wurde die Signatur bereits im Konstruktor geprüft.
    Ok(())
}

// ANCHOR[contextra-license:STEP-5-EXPIRATION]
/// Schritt 5: Ablaufdatumsprüfung über den Zeit-Port ([`Clock`]).
///
/// Liest die aktuelle Zeit über `clock.now_unix_nanos()` (Regel 3: kein direkter OS-Zugriff).
///
/// **Fehler:** Gibt [`StepVerificationError::Expired`] zurück, falls `current_time >= expires_at`.
///
/// # Monotonie-Hinweis
/// Wanduhren (`SystemClock`) können z. B. durch NTP-Sprünge zurückgestellt werden. In Offline-Deployments
/// muss eine vertrauenswürdige Zeitquelle gewählt werden.
pub fn verify_step_5_expiration(
    activation: Option<&SignedActivation>,
    payload: Option<&LicensePayload>,
    clock: &dyn Clock,
) -> Result<(), StepVerificationError> {
    let expires_at = if let Some(act) = activation {
        act.expires_at_unix
    } else if let Some(pld) = payload {
        pld.expires_at.unwrap_or(i64::MAX)
    } else {
        i64::MAX
    };

    let now_secs = (clock.now_unix_nanos() / 1_000_000_000) as i64;
    if now_secs >= expires_at {
        return Err(StepVerificationError::Expired {
            expires_at,
            current_time: now_secs,
        });
    }

    Ok(())
}

// ANCHOR[contextra-license:STEP-6-RING-PERMISSION]
/// Schritt 6: Prüfung der Ring-Berechtigung.
///
/// Prüft, ob der konkret angeforderte Feature-Ring in der Lizenz enthalten ist.
/// Es gibt keine automatische implizite Ring-Hierarchie; jeder Ring muss explizit erlaubt sein.
///
/// **Fehler:** Gibt [`StepVerificationError::RingNotPermitted`] zurück, wenn der Ring fehlt.
pub fn verify_step_6_ring_permission(
    activation: Option<&SignedActivation>,
    payload: Option<&LicensePayload>,
    requested_ring: FeatureRing,
) -> Result<(), StepVerificationError> {
    if let Some(act) = activation {
        if act.ring != requested_ring {
            return Err(StepVerificationError::RingNotPermitted(requested_ring));
        }
    } else if let Some(pld) = payload {
        if !pld.allowed_rings.contains(&requested_ring) {
            return Err(StepVerificationError::RingNotPermitted(requested_ring));
        }
    } else {
        return Err(StepVerificationError::MissingActivation(requested_ring));
    }
    Ok(())
}

// ANCHOR[contextra-license:STEP-7-SUCCESS]
/// Schritt 7: Bestätigung des erfolgreichen Verifikationsabschlusses.
///
/// Dient als finaler Schlusspunkt der 7-stufigen Verifikationskette.
pub fn verify_step_7_success() -> Result<(), StepVerificationError> {
    Ok(())
}

// ANCHOR[contextra-license:STRUCT-SIGNED-LICENSE-GATE]
/// Kryptografisches Lizenz-Gate zur Durchsetzung von Feature-Ringen.
///
/// Verifiziert Ed25519-Signaturen über Aktivierungsdatensätze ([`SignedActivation`])
/// oder signierte Payloads ([`LicensePayload`]) und implementiert den [`LicenseGate`]-Port.
#[derive(Clone)]
pub struct SignedLicenseGate {
    verifying_key: Option<VerifyingKey>,
    activation: Option<SignedActivation>,
    license_payload: Option<LicensePayload>,
    local_installation_id: Option<[u8; 32]>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for SignedLicenseGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignedLicenseGate")
            .field("verifying_key", &self.verifying_key)
            .field("activation", &self.activation)
            .field("license_payload", &self.license_payload)
            .field("local_installation_id", &self.local_installation_id)
            .finish()
    }
}

impl SignedLicenseGate {
    // ANCHOR[contextra-license:GATE-NO-ACTIVATION]
    /// Erzeugt ein leeres Gate ohne aktive Lizenz (nur der `Fast`-Ring ist freigeschaltet).
    pub fn no_activation() -> Self {
        Self {
            verifying_key: None,
            activation: None,
            license_payload: None,
            local_installation_id: None,
            clock: Arc::new(SystemClock::new()),
        }
    }

    // ANCHOR[contextra-license:GATE-FROM-ACTIVATION]
    /// Konstruiert ein Gate aus einem [`SignedActivation`]-Datensatz und dem öffentlichen Ed25519-Schlüssel.
    /// Nutzt standardmäßig die [`SystemClock`].
    pub fn from_activation(activation: SignedActivation, verifying_key: VerifyingKey) -> Self {
        Self::from_activation_with_clock(activation, verifying_key, Arc::new(SystemClock::new()))
    }

    // ANCHOR[contextra-license:GATE-FROM-ACTIVATION-CLOCK]
    /// Konstruiert ein Gate aus einem [`SignedActivation`]-Datensatz mit injiziertem [`Clock`]-Port.
    pub fn from_activation_with_clock(
        activation: SignedActivation,
        verifying_key: VerifyingKey,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            verifying_key: Some(verifying_key),
            activation: Some(activation),
            license_payload: None,
            local_installation_id: None,
            clock,
        }
    }

    // ANCHOR[contextra-license:GATE-FROM-SIGNED-PAYLOAD]
    /// Konstruiert ein Gate durch Signaturprüfung eines Roh-Payloads mit Standardeinstellungen.
    ///
    /// # Fehler
    /// Gibt [`LicenseError::InvalidSignature`] zurück, falls Signatur ungültig oder Deserialisierung fehlschlägt.
    pub fn from_signed_payload(
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key: VerifyingKey,
    ) -> Result<Self, LicenseError> {
        Self::from_signed_payload_with_clock_and_opts(
            payload_bytes,
            signature,
            verifying_key,
            Arc::new(SystemClock::new()),
            SignedLicenseOptions::default(),
        )
    }

    // ANCHOR[contextra-license:GATE-FROM-SIGNED-PAYLOAD-CLOCK]
    /// Konstruiert ein Gate aus signierten Payload-Bytes mit injiziertem [`Clock`]-Port.
    pub fn from_signed_payload_with_clock(
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key: VerifyingKey,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, LicenseError> {
        Self::from_signed_payload_with_clock_and_opts(
            payload_bytes,
            signature,
            verifying_key,
            clock,
            SignedLicenseOptions::default(),
        )
    }

    // ANCHOR[contextra-license:GATE-FROM-SIGNED-PAYLOAD-OPTS]
    /// Konstruiert ein Gate aus signierten Payload-Bytes mit Optionen und Zeit-Port.
    ///
    /// **Ablauf:**
    /// 1. Prüft, ob `payload_bytes` die Obergrenze `options.max_payload_bytes` überschreitet.
    /// 2. Verifiziert die Ed25519-Signatur über die Bytes mit `verifying_key`.
    /// 3. Deserialisiert die Bytes strikt mit Bincode (Trailing Bytes werden abgelehnt).
    /// 4. Falls Deserialisierung fehlschlägt und `allow_legacy` aktiv ist, versucht Fallback auf Legacy-Schema.
    /// 5. Gibt die initialisierte Instanz zurück oder [`LicenseError::InvalidSignature`].
    pub fn from_signed_payload_with_clock_and_opts(
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key: VerifyingKey,
        clock: Arc<dyn Clock>,
        options: SignedLicenseOptions,
    ) -> Result<Self, LicenseError> {
        if payload_bytes.len() > options.max_payload_bytes {
            return Err(LicenseError::InvalidSignature);
        }

        let sig = Signature::from_bytes(signature);
        verifying_key
            .verify(payload_bytes, &sig)
            .map_err(|_| LicenseError::InvalidSignature)?;

        let bincode_opts = bincode::options()
            .with_fixint_encoding()
            .reject_trailing_bytes();

        if let Ok(license_payload) = bincode_opts.deserialize::<LicensePayload>(payload_bytes) {
            return Ok(Self {
                verifying_key: Some(verifying_key),
                activation: None,
                license_payload: Some(license_payload),
                local_installation_id: None,
                clock,
            });
        }

        if options.allow_legacy {
            if let Ok(legacy) = bincode_opts.deserialize::<LegacyLicensePayload>(payload_bytes) {
                return Ok(Self {
                    verifying_key: Some(verifying_key),
                    activation: None,
                    license_payload: Some(legacy.into()),
                    local_installation_id: None,
                    clock,
                });
            }
        }

        Err(LicenseError::InvalidSignature)
    }

    // ANCHOR[contextra-license:GATE-WITH-LOCAL-ID]
    /// Bindet einen lokalen 32-Byte-Installations-Hash an diese Gate-Instanz.
    pub fn with_local_installation_id(mut self, id_hash: [u8; 32]) -> Self {
        self.local_installation_id = Some(id_hash);
        self
    }

    // ANCHOR[contextra-license:GATE-GETTERS]
    /// Gibt eine Referenz auf das geprüfte [`LicensePayload`] zurück, falls vorhanden.
    pub fn license_payload(&self) -> Option<&LicensePayload> {
        self.license_payload.as_ref()
    }

    /// Gibt eine Referenz auf den geprüften [`SignedActivation`]-Datensatz zurück, falls vorhanden.
    pub fn activation(&self) -> Option<&SignedActivation> {
        self.activation.as_ref()
    }

    /// Gibt eine Referenz auf den öffentlichen Ed25519-Schlüssel zurück, falls hinterlegt.
    pub fn verifying_key(&self) -> Option<&VerifyingKey> {
        self.verifying_key.as_ref()
    }

    // ANCHOR[contextra-license:GATE-STEP-CHECKS]
    /// Führt Schritt 1 (Fast Ring Bypass) auf dieser Instanz aus.
    pub fn check_step_1_fast_ring(&self, ring: FeatureRing) -> bool {
        verify_step_1_fast_ring(ring)
    }

    /// Führt Schritt 2 (Vorhandenseinsprüfung) auf dieser Instanz aus.
    pub fn check_step_2_presence(&self, ring: FeatureRing) -> Result<(), StepVerificationError> {
        verify_step_2_presence(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            ring,
        )
    }

    /// Führt Schritt 3 (Installationsbindung) auf dieser Instanz aus.
    pub fn check_step_3_installation(
        &self,
        ring: FeatureRing,
    ) -> Result<(), StepVerificationError> {
        verify_step_3_installation(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            self.local_installation_id.as_ref(),
            ring,
        )
    }

    /// Führt Schritt 4 (Signaturprüfung) auf dieser Instanz aus.
    pub fn check_step_4_signature(&self) -> Result<(), StepVerificationError> {
        verify_step_4_signature(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            self.verifying_key.as_ref(),
        )
    }

    /// Führt Schritt 5 (Ablaufprüfung via Zeit-Port) auf dieser Instanz aus.
    pub fn check_step_5_expiration(&self) -> Result<(), StepVerificationError> {
        verify_step_5_expiration(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            self.clock.as_ref(),
        )
    }

    /// Führt Schritt 6 (Ring-Berechtigung) auf dieser Instanz aus.
    pub fn check_step_6_ring_permission(
        &self,
        ring: FeatureRing,
    ) -> Result<(), StepVerificationError> {
        verify_step_6_ring_permission(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            ring,
        )
    }

    /// Führt Schritt 7 (Erfolgsbestätigung) auf dieser Instanz aus.
    pub fn check_step_7_success(&self) -> Result<(), StepVerificationError> {
        verify_step_7_success()
    }

    // ANCHOR[contextra-license:GATE-CREATE-TEST-PAYLOAD]
    /// Erzeugt Testdaten (Payload-Bytes, Signatur und Verifying-Key-Bytes) für Unit- und Integrationstests.
    pub fn create_test_signed_payload(
        rings: Vec<FeatureRing>,
        expires_at: Option<i64>,
    ) -> (Vec<u8>, [u8; 64], [u8; 32]) {
        let secret_bytes = [42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key_bytes = signing_key.verifying_key().to_bytes();

        let payload = LicensePayload {
            tenant_id: TenantId::SYSTEM,
            allowed_rings: rings,
            expires_at,
            feature_flags: BTreeMap::new(),
            installation_id_hash: None,
        };

        let payload_bytes = bincode::serialize(&payload).unwrap_or_default();
        let signature = signing_key.sign(&payload_bytes).to_bytes();

        (payload_bytes, signature, verifying_key_bytes)
    }
}

// ANCHOR[contextra-license:IMPL-LICENSE-GATE]
impl LicenseGate for SignedLicenseGate {
    /// Überprüft, ob der angeforderte Feature-Ring nach dem 7-stufigen Fail-Closed-Verfahren freigeschaltet ist.
    ///
    /// **Ablauf:**
    /// 1. **Schritt 1:** Fast Ring Bypass (INV-LICENSE-2). Ist `ring == FeatureRing::Fast`, sofort `Ok(())`.
    /// 2. **Schritt 2:** Prüfen, ob Aktivierung oder Payload vorhanden ist.
    /// 3. **Schritt 3:** Prüfen der lokalen Installations-ID in konstanter Zeit ([`constant_time_eq_32`]).
    /// 4. **Schritt 4:** Kryptografische Prüfung der Ed25519-Signatur.
    /// 5. **Schritt 5:** Prüfung des Ablaufdatums gegen den [`Clock`]-Port.
    /// 6. **Schritt 6:** Prüfung, ob der Ring in den erlaubten Ringen enthalten ist.
    /// 7. **Schritt 7:** Bestätigung und Rückgabe von `Ok(())`.
    ///
    /// Scheitert irgendein Schritt ab Schritt 2, bricht die Prüfung sofort fail-closed ab.
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {
        // (1) Step 1: Fast Ring Bypass - INVARIANTE INV-LICENSE-2
        if verify_step_1_fast_ring(ring) {
            return Ok(());
        }

        // (2) Step 2: Missing activation check
        verify_step_2_presence(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            ring,
        )
        .map_err(|e| e.into_license_error())?;

        // (3) Step 3: Installation ID hash mismatch check (constant time, fail-closed)
        verify_step_3_installation(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            self.local_installation_id.as_ref(),
            ring,
        )
        .map_err(|e| e.into_license_error())?;

        // (4) Step 4: Invalid signature check
        verify_step_4_signature(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            self.verifying_key.as_ref(),
        )
        .map_err(|e| e.into_license_error())?;

        // (5) Step 5: Expiration check using clock port (P28)
        verify_step_5_expiration(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            self.clock.as_ref(),
        )
        .map_err(|e| e.into_license_error())?;

        // (6) Step 6: Ring level requirement check
        verify_step_6_ring_permission(
            self.activation.as_ref(),
            self.license_payload.as_ref(),
            ring,
        )
        .map_err(|e| e.into_license_error())?;

        // (7) Step 7: Success confirmation
        verify_step_7_success().map_err(|e| e.into_license_error())
    }
}
