// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: RFC 9162 Transparenzlog (Merkle Tree, Inklusions- und Konsistenzbeweise) für gesalzen Löschnachweis-IDs.
// INVARIANTEN: Zero-Panic, SHA-256 Präfixe (0x00 Blatt, 0x01 Knoten), SDM 60 / EDPB 02/2025 Konformität (nur gesalzene Hashes).

use crate::error::CryptoError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Prefix byte for leaf hashes per RFC 9162 §2.1.
pub const LEAF_PREFIX: u8 = 0x00;

/// Prefix byte for internal node hashes per RFC 9162 §2.1.
pub const NODE_PREFIX: u8 = 0x01;

/// Inklusionsbeweis für ein einzelnes Blatt im Merkle-Baum (RFC 9162 §2.1.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InclusionProof {
    /// Index des Blattes im Baum (0..tree_size-1).
    pub leaf_index: usize,
    /// Gesamtanzahl der Blätter im Baum.
    pub tree_size: usize,
    /// Geschwister-Hashes entlang des Pfades zur Wurzel.
    pub audit_path: Vec<[u8; 32]>,
}

/// Konsistenzbeweis zwischen zwei Zuständen des Transparenzlogs (RFC 9162 §2.1.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsistencyProof {
    /// Baumgröße des älteren Zustands.
    pub first_tree_size: usize,
    /// Baumgröße des neueren Zustands.
    pub second_tree_size: usize,
    /// Hashes zum Nachweis der Append-Only-Eigenschaft.
    pub audit_path: Vec<[u8; 32]>,
}

/// Erzeugt den gesalzenen Hash einer Löschnachweis-ID (SDM 60, EDPB 02/2025).
/// Klartext-IDs dürfen NIEMALS im Log gespeichert werden.
pub fn hash_salted_proof_id(proof_id: &[u8], salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(proof_id);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// RFC 9162 Merkle-Baum für Transparenzlogs.
#[derive(Debug, Clone)]
pub struct TransparencyLog {
    /// Gesalzene Blatt-Hashes.
    leaf_hashes: Vec<[u8; 32]>,
}

impl TransparencyLog {
    /// Erzeugt ein neues Transparenzlog mit gesalzenen Blatt-Hashes.
    pub fn new(leaf_hashes: Vec<[u8; 32]>) -> Self {
        Self { leaf_hashes }
    }

    /// Gibt die Anzahl der Blätter im Log zurück.
    pub fn tree_size(&self) -> usize {
        self.leaf_hashes.len()
    }

    /// Berechnet die Wurzel des Merkle-Baums (RFC 9162 §2.1.1).
    pub fn root_hash(&self) -> [u8; 32] {
        mth(&self.leaf_hashes)
    }

    /// Erzeugt einen Inklusionsbeweis für ein bestimmtes Blatt (RFC 9162 §2.1.3).
    pub fn inclusion_proof(&self, leaf_index: usize) -> Result<InclusionProof, CryptoError> {
        let n = self.leaf_hashes.len();
        if n == 0 {
            return Err(CryptoError::InvalidInput(
                "Cannot generate inclusion proof for an empty tree".to_string(),
            ));
        }
        if leaf_index >= n {
            return Err(CryptoError::InvalidInput(format!(
                "Leaf index {} out of bounds for tree size {}",
                leaf_index, n
            )));
        }

        let audit_path = path(leaf_index, &self.leaf_hashes);

        Ok(InclusionProof {
            leaf_index,
            tree_size: n,
            audit_path,
        })
    }

    /// Erzeugt einen Konsistenzbeweis zwischen zwei Baumgrößen (RFC 9162 §2.1.4).
    pub fn consistency_proof(
        &self,
        first_tree_size: usize,
    ) -> Result<ConsistencyProof, CryptoError> {
        let second_tree_size = self.leaf_hashes.len();
        if first_tree_size == 0 || first_tree_size > second_tree_size {
            return Err(CryptoError::InvalidInput(format!(
                "Invalid first tree size {} for current log size {}",
                first_tree_size, second_tree_size
            )));
        }

        let audit_path = subproof(first_tree_size, &self.leaf_hashes[..second_tree_size], true);

        Ok(ConsistencyProof {
            first_tree_size,
            second_tree_size,
            audit_path,
        })
    }
}

/// Merkle Tree Hash (MTH) Berechnung gemäß RFC 9162 §2.1.1.
pub fn mth(leaves: &[[u8; 32]]) -> [u8; 32] {
    let n = leaves.len();
    if n == 0 {
        let hasher = Sha256::new();
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        return out;
    }
    if n == 1 {
        let mut hasher = Sha256::new();
        hasher.update([LEAF_PREFIX]);
        hasher.update(&leaves[0]);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        return out;
    }

    let k = largest_power_of_two_less_than(n);
    let left = mth(&leaves[..k]);
    let right = mth(&leaves[k..]);

    let mut hasher = Sha256::new();
    hasher.update([NODE_PREFIX]);
    hasher.update(&left);
    hasher.update(&right);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Größte Zweierpotenz kleiner als `n` (für `n > 1`).
pub fn largest_power_of_two_less_than(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    let mut k = 1;
    while k < n {
        let next = k << 1;
        if next >= n {
            break;
        }
        k = next;
    }
    k
}

/// RFC 9162 §2.1.3 PATH(m, D[n]) Berechnungs-Hilfsfunktion.
fn path(m: usize, leaves: &[[u8; 32]]) -> Vec<[u8; 32]> {
    let n = leaves.len();
    if n <= 1 {
        return Vec::new();
    }

    let k = largest_power_of_two_less_than(n);
    if m < k {
        let mut p = path(m, &leaves[..k]);
        p.push(mth(&leaves[k..]));
        p
    } else {
        let mut p = path(m - k, &leaves[k..]);
        p.push(mth(&leaves[..k]));
        p
    }
}

/// RFC 9162 §2.1.4 SUBPROOF(m, D[n], b) Berechnungs-Hilfsfunktion.
fn subproof(m: usize, leaves: &[[u8; 32]], b: bool) -> Vec<[u8; 32]> {
    let n = leaves.len();
    if m == n {
        if b {
            Vec::new()
        } else {
            vec![mth(leaves)]
        }
    } else {
        let k = largest_power_of_two_less_than(n);
        if m <= k {
            let mut p = subproof(m, &leaves[..k], b);
            p.push(mth(&leaves[k..]));
            p
        } else {
            let mut p = subproof(m - k, &leaves[k..], false);
            p.push(mth(&leaves[..k]));
            p
        }
    }
}

/// Verifiziert einen Inklusionsbeweis für ein gegebenes Blatt (RFC 9162 §2.1.3).
pub fn verify_inclusion_proof(
    leaf_hash: &[u8; 32],
    proof: &InclusionProof,
    expected_root: &[u8; 32],
) -> Result<bool, CryptoError> {
    if proof.tree_size == 0 || proof.leaf_index >= proof.tree_size {
        return Err(CryptoError::InvalidInput(
            "Invalid inclusion proof tree size or index".to_string(),
        ));
    }

    let calculated_root = root_from_inclusion_path(
        proof.leaf_index,
        proof.tree_size,
        leaf_hash,
        &proof.audit_path,
    )?;

    Ok(&calculated_root == expected_root)
}

/// Rekonstruiert die Wurzel aus einem Inklusionspfad (RFC 9162 §2.1.3).
fn root_from_inclusion_path(
    m: usize,
    n: usize,
    leaf_hash: &[u8; 32],
    audit_path: &[[u8; 32]],
) -> Result<[u8; 32], CryptoError> {
    if n == 1 {
        if !audit_path.is_empty() {
            return Err(CryptoError::InvalidInput(
                "Audit path must be empty for tree size 1".to_string(),
            ));
        }
        let mut hasher = Sha256::new();
        hasher.update([LEAF_PREFIX]);
        hasher.update(leaf_hash);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        return Ok(out);
    }

    if audit_path.is_empty() {
        return Err(CryptoError::InvalidInput(
            "Audit path empty for tree size > 1".to_string(),
        ));
    }

    let k = largest_power_of_two_less_than(n);
    let sibling = audit_path[audit_path.len() - 1];
    let remaining_path = &audit_path[..audit_path.len() - 1];

    if m < k {
        let left = root_from_inclusion_path(m, k, leaf_hash, remaining_path)?;
        let right = sibling;

        let mut hasher = Sha256::new();
        hasher.update([NODE_PREFIX]);
        hasher.update(&left);
        hasher.update(&right);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        Ok(out)
    } else {
        let left = sibling;
        let right = root_from_inclusion_path(m - k, n - k, leaf_hash, remaining_path)?;

        let mut hasher = Sha256::new();
        hasher.update([NODE_PREFIX]);
        hasher.update(&left);
        hasher.update(&right);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        Ok(out)
    }
}

/// Verifiziert einen Konsistenzbeweis zwischen zwei Baumgrößen (RFC 9162 §2.1.4).
pub fn verify_consistency_proof(
    proof: &ConsistencyProof,
    first_root: &[u8; 32],
    second_root: &[u8; 32],
) -> Result<bool, CryptoError> {
    let first = proof.first_tree_size;
    let second = proof.second_tree_size;

    if first == 0 || first > second {
        return Err(CryptoError::InvalidInput(
            "Invalid tree size bounds in consistency proof".to_string(),
        ));
    }

    if first == second {
        if !proof.audit_path.is_empty() {
            return Err(CryptoError::InvalidInput(
                "Audit path must be empty when tree sizes match".to_string(),
            ));
        }
        return Ok(first_root == second_root);
    }

    let mut path = proof.audit_path.clone();
    let (calc_first, calc_second) =
        eval_consistency_path(first, second, &mut path, true, first_root)?;

    if !path.is_empty() {
        return Err(CryptoError::InvalidInput(
            "Unused elements remaining in consistency proof audit path".to_string(),
        ));
    }

    Ok(&calc_first == first_root && &calc_second == second_root)
}

fn eval_consistency_path(
    m: usize,
    n: usize,
    path: &mut Vec<[u8; 32]>,
    b: bool,
    expected_first: &[u8; 32],
) -> Result<([u8; 32], [u8; 32]), CryptoError> {
    if m == n {
        if b {
            Ok((*expected_first, *expected_first))
        } else {
            let h = path.pop().ok_or_else(|| {
                CryptoError::InvalidInput("Consistency proof path truncated".to_string())
            })?;
            Ok((h, h))
        }
    } else {
        let k = largest_power_of_two_less_than(n);
        let sibling = path.pop().ok_or_else(|| {
            CryptoError::InvalidInput("Consistency proof path truncated".to_string())
        })?;

        if m <= k {
            let (c_first, left_sec) = eval_consistency_path(m, k, path, b, expected_first)?;
            let right_sec = sibling;

            let mut hasher = Sha256::new();
            hasher.update([NODE_PREFIX]);
            hasher.update(&left_sec);
            hasher.update(&right_sec);
            let result = hasher.finalize();
            let mut c_second = [0u8; 32];
            c_second.copy_from_slice(&result);

            Ok((c_first, c_second))
        } else {
            let (left_sec, right_sec) =
                eval_consistency_path(m - k, n - k, path, false, expected_first)?;
            let left_first = sibling;

            let mut hasher = Sha256::new();
            hasher.update([NODE_PREFIX]);
            hasher.update(&left_first);
            hasher.update(&left_sec);
            let result = hasher.finalize();
            let mut c_first = [0u8; 32];
            c_first.copy_from_slice(&result);

            let mut hasher2 = Sha256::new();
            hasher2.update([NODE_PREFIX]);
            hasher2.update(&left_first);
            hasher2.update(&right_sec);
            let result2 = hasher2.finalize();
            let mut c_second = [0u8; 32];
            c_second.copy_from_slice(&result2);

            Ok((c_first, c_second))
        }
    }
}
