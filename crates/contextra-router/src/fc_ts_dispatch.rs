//! FC-TS-Profilauswahl für contextra-router (§21.3, AK-18). Eigenständiges Modul, analog im Aufbau
//! zu `router::select_profile_cascade`, aber ohne Abhängigkeit von `router.rs`/`profile.rs`.

use contextra_adapt::{FcTsArmSet, FcTsError, FcTsRng, SplitMix64};

/// Fehler der FC-TS-Profilauswahl für contextra-router.
#[derive(Debug, thiserror::Error)]
pub enum FcTsDispatchError {
    #[error("FC-TS arm set is empty or misconfigured: {0}")]
    ArmSet(#[from] FcTsError),
    #[error("profile/arm count mismatch: {profiles} profiles vs {arms} arms")]
    ProfileArmMismatch { profiles: usize, arms: usize },
}

/// Wählt einen Profil-Index über FC-TS-Argmax-Sampling aus `arm_set` für den gegebenen Kontextvektor.
/// `profile_names[i]` MUSS `arm_set.arms[i]` entsprechen (aufrufende Seite garantiert 1:1-Mapping).
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

/// Deterministischer Default-RNG-Konstruktor für reproduzierbare FC-TS-Auswahl bei gegebenem Seed
/// (z. B. abgeleitet aus einer Request-ID) — kapselt `SplitMix64`, damit Aufrufer nicht direkt an
/// `contextra_adapt` gekoppelt sein müssen.
pub fn deterministic_fc_ts_rng(seed: u64) -> impl FcTsRng {
    SplitMix64::new(seed)
}
