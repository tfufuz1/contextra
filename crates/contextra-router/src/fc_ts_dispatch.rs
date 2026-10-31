//! # FC-TS-Profilauswahl für contextra-router (§21.3, AK-18)
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Flow-Corrected Thompson Sampling (FC-TS) zur probabilistischen Profilauswahl.
//! Eigenständiges Modul, analog im Aufbau zu `router::select_profile_cascade`, aber ohne direkte
//! Abhängigkeit von internen Strukturen in `router/` oder `profile.rs`.
//!
//! **Hintergrund:**
//! Thompson Sampling wählt Aktionen basierend auf Posterior-Verteilungen von Belohnungsschätzern aus.
//! Die Flow-Korrektur passt die Zuteilung an veränderliche Systembedingungen an.
//!
//! **Abhängigkeiten:**
//! - Upstream: `contextra_adapt::{FcTsArmSet, FcTsError, FcTsRng, SplitMix64}`.
//! - Downstream: Externe Aufrufer und Test-Suiten für Flow-Corrected Dispatching.

use contextra_adapt::{FcTsArmSet, FcTsError, FcTsRng, SplitMix64};

// ANCHOR[contextra-router:FCTS-001]
/// Fehler der FC-TS-Profilauswahl für contextra-router.
#[derive(Debug, thiserror::Error)]
pub enum FcTsDispatchError {
    /// Das FC-TS-Arm-Set ist leer oder ungültig konfiguriert.
    #[error("FC-TS arm set is empty or misconfigured: {0}")]
    ArmSet(#[from] FcTsError),
    /// Anzahl der übergebenen Profilnamen stimmt nicht mit der Anzahl der Arme überein.
    #[error("profile/arm count mismatch: {profiles} profiles vs {arms} arms")]
    ProfileArmMismatch { profiles: usize, arms: usize },
}

// ANCHOR[contextra-router:FCTS-002]
/// Wählt einen Profil-Index über FC-TS-Argmax-Sampling aus `arm_set` für den gegebenen Kontextvektor.
///
/// `profile_names[i]` MUSS `arm_set.arms[i]` entsprechen (die aufrufende Seite garantiert ein 1:1-Mapping).
///
/// # Ablauf
/// 1. Validierung der Längenübereinstimmung von Profilnamen und Armen.
/// 2. Ziehung des besten Arms via Argmax-Sampling aus dem Posterior über den bereitgestellten PRNG (`rng`).
/// 3. Rückgabe des Index und des geklonten Profilnamens.
///
/// # Vorbedingungen
/// - `profile_names.len() == arm_set.arms.len()`.
/// - `context` hat die Dimension, die von den Armen erwartet wird.
///
/// # Nachbedingungen
/// Liefert das ausgewählte Profil-Paar `(Index, Name)` zurück.
///
/// # Fehler
/// - [`FcTsDispatchError::ProfileArmMismatch`]: Wenn Profilanzahl und Armanzahl divergieren.
/// - [`FcTsDispatchError::ArmSet`]: Wenn das Arm-Set ungültig ist oder die Berechnung fehlschlägt.
///
/// # Nebenwirkungen
/// Mutiert den Zustand des Zufallszahlengenerators `rng`. Keine I/O-Effekte.
// NOTE: Zufall wird streng über den `rng`-Parameter injiziert (Port-Entwurf, kein unkontrollierter System-Zufall).
pub fn select_profile_fc_ts(
    profile_names: &[String],
    arm_set: &FcTsArmSet,
    context: &[f32],
    rng: &mut dyn FcTsRng,
) -> Result<(usize, String), FcTsDispatchError> {
    if profile_names.len() != arm_set.arms.len() {
        return Err(FcTsDispatchError::ProfileArmMismatch {
            profiles: profile_names.len(),
            arms: arm_set.arms.len(),
        });
    }
    let idx = arm_set.select_arm(context, rng)? as usize;
    Ok((idx, profile_names[idx].clone()))
}

// ANCHOR[contextra-router:FCTS-003]
/// Deterministischer Default-RNG-Konstruktor für reproduzierbare FC-TS-Auswahl bei gegebenem Seed.
///
/// Kapselt den `SplitMix64`-PRNG (Pseudo-Random Number Generator), damit Aufrufer nicht direkt an
/// interne Implementierungsdetails von `contextra_adapt` gekoppelt sein müssen.
///
/// # Parameter
/// - `seed`: 64-Bit-Ganzzahl (z. B. abgeleitet aus einer deterministischen Request-ID).
///
/// # Rückgabe
/// Eine Implementierung von [`FcTsRng`], die deterministische Zufallszahlen liefert.
// NOTE: Wichtig für reproduzierbare Replays und Tests.
pub fn deterministic_fc_ts_rng(seed: u64) -> impl FcTsRng {
    SplitMix64::new(seed)
}
