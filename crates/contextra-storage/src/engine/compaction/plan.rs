//! # Compaction Planer (rein)
//!
//! SPEC §12.2:
//! Reine Funktion zur Kompaktionsplanung:
//! - Auslöser: L0-Anzahl >= 4 (alle L0 plus überlappende L1-Runs)
//! - Tiefere Level: Größe > target(level) (Faktor 10 je Level)
//! - Ausgabe bei target_run_bytes teilen, bevorzugt an Präfixgrenzen (SD-11, §12.6)
//! - Planer-Eigenschaft: Auf Level >= 1 keine überlappenden Ausgaben.
//! Keine Integer-Casts (M-02 = 0), keine Panics (M-08 = 0).

use crate::catalog::state::CatalogState;
use crate::catalog::types::{CatalogFileKind, FileMeta};
use std::collections::HashSet;

/// Parameter für die Kompaktionsplanung nach SPEC §12.2.
#[derive(Debug, Clone)]
pub struct CompactionParams {
    /// L0-Trigger: Mindestanzahl von L0-Runs (Standard: 4).
    pub l0_trigger: usize,
    /// Basisgröße für Level 1 in Bytes (Standard: 10 MiB).
    pub base_level_bytes: u64,
    /// Multiplikator je tieferem Level (Standard: 10).
    pub level_multiplier: u64,
    /// Maximale Anzahl von Leveln (L0 bis L6 = 7).
    pub max_levels: u8,
    /// Zielgröße je Ausgabe-Run (Standard: 64 MiB).
    pub target_run_bytes: u64,
    /// Länge des Lokalitätspräfixes zur Teilung (0 = deaktiviert).
    pub locality_prefix_len: u8,
}

impl Default for CompactionParams {
    fn default() -> Self {
        Self {
            l0_trigger: 4,
            base_level_bytes: 10 * 1024 * 1024,
            level_multiplier: 10,
            max_levels: 7,
            target_run_bytes: 64 * 1024 * 1024,
            locality_prefix_len: 0,
        }
    }
}

/// Menge der aktiven Datei-Reservierungen (Claims) nach SPEC §16.5.
#[derive(Debug, Clone, Default)]
pub struct Claims {
    claimed: HashSet<(CatalogFileKind, u64)>,
}

impl Claims {
    /// Erstellt eine neue leere Claims-Menge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fügt eine reservierte Datei hinzu.
    pub fn claim(&mut self, kind: CatalogFileKind, id: u64) {
        self.claimed.insert((kind, id));
    }

    /// Prüft, ob eine Datei aktuell beansprucht ist.
    pub fn is_claimed(&self, kind: CatalogFileKind, id: u64) -> bool {
        self.claimed.contains(&(kind, id))
    }
}

/// Ein deterministischer Plan für einen Kompaktionslauf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionPlan {
    /// Zugehöriger Space.
    pub space: u16,
    /// Quell-Level.
    pub source_level: u8,
    /// Ziel-Level.
    pub target_level: u8,
    /// Alle Eingabe-Runs (Quell- und überlappende Ziel-Runs).
    pub input_runs: Vec<FileMeta>,
    /// Kleinster Schlüssel der Eingaben.
    pub min_key: Vec<u8>,
    /// Größter Schlüssel der Eingaben.
    pub max_key: Vec<u8>,
    /// Vorgesehene Zielgröße der Ausgabe-Runs.
    pub target_run_bytes: u64,
    /// Untergrenze für die Versionenaufbewahrung.
    pub floor: u64,
    /// Zeigt an, ob Tombstones sicher physisch entfernt werden dürfen.
    pub can_drop_tombstones: bool,
}

/// Prüft, ob zwei geschlossene Schlüsselbereiche [min_a, max_a] und [min_b, max_b] überlappen.
pub fn key_ranges_overlap(min_a: &[u8], max_a: &[u8], min_b: &[u8], max_b: &[u8]) -> bool {
    min_a <= max_b && min_b <= max_a
}

/// Reine Funktion zur Planung des nächsten Kompaktionsauftrags (SPEC §12.2).
pub fn plan(
    state: &CatalogState,
    params: &CompactionParams,
    floor: u64,
    claims: &Claims,
) -> Option<CompactionPlan> {
    // 1. Sammle alle Runs gruppiert nach (space, level)
    let mut runs_by_space_level: std::collections::HashMap<(u16, u8), Vec<FileMeta>> =
        std::collections::HashMap::new();

    for meta in state.files.values() {
        if meta.kind == CatalogFileKind::Run {
            runs_by_space_level
                .entry((meta.space, meta.level))
                .or_default()
                .push(meta.clone());
        }
    }

    // Sortiere deterministisch nach Space
    let mut spaces: Vec<u16> = state.spaces.keys().copied().collect();
    spaces.sort_unstable();

    for &space in &spaces {
        // --- Prüfe Auslöser 1: L0-Anzahl >= l0_trigger (SPEC §12.2) ---
        let l0_runs = runs_by_space_level
            .get(&(space, 0))
            .cloned()
            .unwrap_or_default();

        if l0_runs.len() >= params.l0_trigger {
            // Alle L0-Runs müssen unbeansprucht sein
            let any_l0_claimed = l0_runs
                .iter()
                .any(|r| claims.is_claimed(CatalogFileKind::Run, r.id));
            if !any_l0_claimed {
                // Bestimme Gesamtschlüsselbereich über alle L0-Runs
                let mut min_k: Option<Vec<u8>> = None;
                let mut max_k: Option<Vec<u8>> = None;

                for r in &l0_runs {
                    update_bounds(&mut min_k, &mut max_k, &r.min_key, &r.max_key);
                }

                if let (Some(l0_min), Some(l0_max)) = (min_k, max_k) {
                    // Finde überlappende Runs auf Level 1
                    let l1_runs = runs_by_space_level
                        .get(&(space, 1))
                        .cloned()
                        .unwrap_or_default();

                    let mut overlapping_l1 = Vec::new();
                    let mut l1_conflict = false;

                    for r in l1_runs {
                        if key_ranges_overlap(&l0_min, &l0_max, &r.min_key, &r.max_key) {
                            if claims.is_claimed(CatalogFileKind::Run, r.id) {
                                l1_conflict = true;
                                break;
                            }
                            overlapping_l1.push(r);
                        }
                    }

                    if !l1_conflict {
                        // Erweitere Schlüsselbereich um überlappende L1-Runs
                        let final_min = l0_min;
                        let final_max = l0_max;
                        for r in &overlapping_l1 {
                            update_bounds(
                                &mut Some(final_min.clone()),
                                &mut Some(final_max.clone()),
                                &r.min_key,
                                &r.max_key,
                            );
                        }

                        let mut input_runs = l0_runs;
                        input_runs.extend(overlapping_l1);

                        // Prüfe, ob tiefere Level existieren, die den Bereich überlappen
                        let can_drop = can_drop_tombstones_for_range(
                            &runs_by_space_level,
                            space,
                            1,
                            params.max_levels,
                            &final_min,
                            &final_max,
                        );

                        return Some(CompactionPlan {
                            space,
                            source_level: 0,
                            target_level: 1,
                            input_runs,
                            min_key: final_min,
                            max_key: final_max,
                            target_run_bytes: params.target_run_bytes,
                            floor,
                            can_drop_tombstones: can_drop,
                        });
                    }
                }
            }
        }

        // --- Prüfe Auslöser 2: Tiefere Level (Größe > target(level)) ---
        let mut best_level: Option<(u8, u64, u64)> = None; // (level, current_bytes, target_bytes)

        for lvl in 1..params.max_levels {
            let runs = runs_by_space_level
                .get(&(space, lvl))
                .cloned()
                .unwrap_or_default();
            let current_bytes: u64 = runs.iter().map(|r| r.bytes).sum();

            // target(level) = base_level_bytes * level_multiplier^(level - 1)
            let exp = u32::from(lvl.saturating_sub(1));
            let target_bytes = params
                .base_level_bytes
                .saturating_mul(params.level_multiplier.saturating_pow(exp));

            if current_bytes > target_bytes {
                let score = current_bytes.saturating_mul(100) / target_bytes.max(1);
                if let Some((_, best_cur, best_tgt)) = best_level {
                    let best_score = best_cur.saturating_mul(100) / best_tgt.max(1);
                    if score > best_score {
                        best_level = Some((lvl, current_bytes, target_bytes));
                    }
                } else {
                    best_level = Some((lvl, current_bytes, target_bytes));
                }
            }
        }

        if let Some((src_lvl, _, _)) = best_level {
            let tgt_lvl = src_lvl.saturating_add(1);
            if tgt_lvl < params.max_levels {
                let src_runs = runs_by_space_level
                    .get(&(space, src_lvl))
                    .cloned()
                    .unwrap_or_default();

                // Wähle den größten noch unbeanspruchten Run des Levels
                let mut candidates: Vec<FileMeta> = src_runs
                    .into_iter()
                    .filter(|r| !claims.is_claimed(CatalogFileKind::Run, r.id))
                    .collect();
                candidates.sort_by(|a, b| b.bytes.cmp(&a.bytes));

                for candidate in candidates {
                    // Überlappende Runs im Ziel-Level finden
                    let tgt_runs = runs_by_space_level
                        .get(&(space, tgt_lvl))
                        .cloned()
                        .unwrap_or_default();

                    let mut overlapping_tgt = Vec::new();
                    let mut conflict = false;

                    for r in tgt_runs {
                        if key_ranges_overlap(
                            &candidate.min_key,
                            &candidate.max_key,
                            &r.min_key,
                            &r.max_key,
                        ) {
                            if claims.is_claimed(CatalogFileKind::Run, r.id) {
                                conflict = true;
                                break;
                            }
                            overlapping_tgt.push(r);
                        }
                    }

                    if !conflict {
                        let mut final_min = candidate.min_key.clone();
                        let mut final_max = candidate.max_key.clone();

                        for r in &overlapping_tgt {
                            if r.min_key < final_min {
                                final_min = r.min_key.clone();
                            }
                            if r.max_key > final_max {
                                final_max = r.max_key.clone();
                            }
                        }

                        let mut input_runs = vec![candidate];
                        input_runs.extend(overlapping_tgt);

                        let can_drop = can_drop_tombstones_for_range(
                            &runs_by_space_level,
                            space,
                            tgt_lvl,
                            params.max_levels,
                            &final_min,
                            &final_max,
                        );

                        return Some(CompactionPlan {
                            space,
                            source_level: src_lvl,
                            target_level: tgt_lvl,
                            input_runs,
                            min_key: final_min,
                            max_key: final_max,
                            target_run_bytes: params.target_run_bytes,
                            floor,
                            can_drop_tombstones: can_drop,
                        });
                    }
                }
            }
        }
    }

    None
}

fn update_bounds(
    min_k: &mut Option<Vec<u8>>,
    max_k: &mut Option<Vec<u8>>,
    r_min: &[u8],
    r_max: &[u8],
) {
    if let Some(cur_min) = min_k {
        if r_min < cur_min.as_slice() {
            *cur_min = r_min.to_vec();
        }
    } else {
        *min_k = Some(r_min.to_vec());
    }

    if let Some(cur_max) = max_k {
        if r_max > cur_max.as_slice() {
            *cur_max = r_max.to_vec();
        }
    } else {
        *max_k = Some(r_max.to_vec());
    }
}

fn can_drop_tombstones_for_range(
    runs: &std::collections::HashMap<(u16, u8), Vec<FileMeta>>,
    space: u16,
    target_lvl: u8,
    max_levels: u8,
    min_k: &[u8],
    max_k: &[u8],
) -> bool {
    let next_lvl = target_lvl.saturating_add(1);
    for lvl in next_lvl..max_levels {
        if let Some(lvl_runs) = runs.get(&(space, lvl)) {
            for r in lvl_runs {
                if key_ranges_overlap(min_k, max_k, &r.min_key, &r.max_key) {
                    return false;
                }
            }
        }
    }
    true
}
