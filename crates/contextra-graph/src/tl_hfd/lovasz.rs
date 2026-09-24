//! Lovász extension and subgradient calculations for TL-HFD (Spec §21.1).

use contextra_core::EntityId;
use std::cmp::Ordering;

/// Truncates hyperedge participants to at most `max_sort_size` top x-value participants plus the min x-value participant.
///
/// # Determinism
/// Deterministic tie-breaking on participant x-values uses ascending `EntityId`.
///
/// # Complexity
/// Time complexity $O(|P| \log |P|)$, where $|P|$ is the number of hyperedge participants.
pub fn truncate_participants(
    participants: &[EntityId],
    get_x: impl Fn(EntityId) -> f32,
    max_sort_size: usize,
) -> Vec<EntityId> {
    if participants.len() <= max_sort_size {
        return participants.to_vec();
    }

    // Sort all participants by x_v descending, tie-breaker: EntityId ascending
    let mut sorted = participants.to_vec();
    sorted.sort_by(|&a, &b| {
        let xa = get_x(a);
        let xb = get_x(b);
        xb.partial_cmp(&xa)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });

    let mut truncated = Vec::with_capacity(max_sort_size + 1);
    truncated.extend_from_slice(&sorted[..max_sort_size]);

    // Find participant with minimum x_v (tie-breaker: EntityId ascending)
    if let Some(&min_p) = sorted.last() {
        if !truncated.contains(&min_p) {
            truncated.push(min_p);
        }
    }

    truncated
}

/// Evaluates Lovász extension $f_e(x) = \max_{u \in e} x_u - \min_{u \in e} x_u$ and subgradient $s_{e, u}(x)$ for participants.
///
/// Returns `(f_e(x), u_max, u_min)` where:
/// - `f_e(x)` is the edge cut cost ($\ge 0.0$).
/// - `u_max` is `Some(EntityId)` if $f_e(x) > 0.0$ and $u_{\max} \neq u_{\min}$, else `None`.
/// - `u_min` is `Some(EntityId)` if $f_e(x) > 0.0$ and $u_{\max} \neq u_{\min}$, else `None`.
///
/// # Complexity
/// Time complexity $O(|e|)$, where $|e|$ is the number of evaluated participants.
pub fn compute_lovasz_extension(
    participants: &[EntityId],
    get_x: impl Fn(EntityId) -> f32,
) -> (f32, Option<EntityId>, Option<EntityId>) {
    if participants.len() < 2 {
        return (0.0, None, None);
    }

    let mut max_val = f32::NEG_INFINITY;
    let mut max_node: Option<EntityId> = None;

    let mut min_val = f32::INFINITY;
    let mut min_node: Option<EntityId> = None;

    for &u in participants {
        let xu = get_x(u);

        // Max: larger x_u, tie-breaker smaller EntityId
        match max_node {
            None => {
                max_val = xu;
                max_node = Some(u);
            }
            Some(curr_max) => {
                if xu > max_val || (xu == max_val && u < curr_max) {
                    max_val = xu;
                    max_node = Some(u);
                }
            }
        }

        // Min: smaller x_u, tie-breaker smaller EntityId
        match min_node {
            None => {
                min_val = xu;
                min_node = Some(u);
            }
            Some(curr_min) => {
                if xu < min_val || (xu == min_val && u < curr_min) {
                    min_val = xu;
                    min_node = Some(u);
                }
            }
        }
    }

    let (u_max, u_min) = match (max_node, min_node) {
        (Some(uma), Some(umi)) => (uma, umi),
        _ => return (0.0, None, None),
    };

    if u_max == u_min || max_val <= min_val {
        (0.0, None, None)
    } else {
        let f_e = max_val - min_val;
        (f_e, Some(u_max), Some(u_min))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lovasz_extension_basic() {
        let e1 = EntityId::new(1);
        let e2 = EntityId::new(2);
        let e3 = EntityId::new(3);

        let participants = vec![e1, e2, e3];
        let get_x = |u: EntityId| match u.inner() {
            1 => 1.0,
            2 => 0.5,
            3 => 0.2,
            _ => 0.0,
        };

        let (f_e, u_max, u_min) = compute_lovasz_extension(&participants, get_x);
        assert!((f_e - 0.8).abs() < 1e-6);
        assert_eq!(u_max, Some(e1));
        assert_eq!(u_min, Some(e3));
    }

    #[test]
    fn test_lovasz_extension_equal_x() {
        let e1 = EntityId::new(1);
        let e2 = EntityId::new(2);

        let participants = vec![e1, e2];
        let get_x = |_| 0.5;

        let (f_e, u_max, u_min) = compute_lovasz_extension(&participants, get_x);
        assert_eq!(f_e, 0.0);
        assert_eq!(u_max, None);
        assert_eq!(u_min, None);
    }

    #[test]
    fn test_lovasz_tie_breaker() {
        let e1 = EntityId::new(1);
        let e2 = EntityId::new(2);
        let e3 = EntityId::new(3);

        // e1 and e2 have same max x = 1.0; e1 < e2 so e1 wins u_max
        // e3 has min x = 0.0
        let participants = vec![e2, e1, e3];
        let get_x = |u: EntityId| match u.inner() {
            1 => 1.0,
            2 => 1.0,
            3 => 0.0,
            _ => 0.0,
        };

        let (f_e, u_max, u_min) = compute_lovasz_extension(&participants, get_x);
        assert_eq!(f_e, 1.0);
        assert_eq!(u_max, Some(e1));
        assert_eq!(u_min, Some(e3));
    }

    #[test]
    fn test_truncation() {
        let p: Vec<EntityId> = (1..=10).map(EntityId::new).collect();
        let get_x = |u: EntityId| u.inner() as f32; // 1.0 .. 10.0

        let truncated = truncate_participants(&p, get_x, 4);
        // Top 4 largest x are 10, 9, 8, 7.
        // Min x is 1. Since 1 is not in top 4, truncated has 5 elements.
        assert_eq!(truncated.len(), 5);
        assert_eq!(
            truncated,
            vec![
                EntityId::new(10),
                EntityId::new(9),
                EntityId::new(8),
                EntityId::new(7),
                EntityId::new(1)
            ]
        );
    }
}
