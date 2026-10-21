# Spec Delta — SESSION W1-06 (contextra-crypto)

## Overview
This spec delta documents the API changes, invariants, and decisions introduced during SESSION W1-06 for `contextra-crypto`.

---

## 1. Canonical Encoding (`canonical.rs`)
- **API**: Introduced `CanonicalWriter`, `CanonicalReader`, `CanonicalTlvEntry`.
- **Wire Format**: TLV encoding with `u16` Little-Endian tag, `u32` Little-Endian length, fixed-width LE integer fields.
- **Enforcement**:
  - Rejection of duplicate tags during encoding or decoding.
  - Rejection of non-canonical tag ordering during stream decoding.
  - Rejection of unknown critical tags (top bit `0x8000` set).
- **Migration**: Signature payload generation in `ProofV4`, `ProofV3`, `ProofV2`, `AuditChainEntry` migrated from `bincode` to `CanonicalWriter`. Reading legacy `bincode` is feature-gated under `legacy-verify`.

---

## 2. Witness Origin & INV-ERA-6
- **API**:
  ```rust
  impl LayerWitness {
      pub fn from_scan(
          scan: ScanOutcome,
          scope_hash: [u8; 32],
          claim: WitClaimText,
          assurance: WitAssurance,
          clock: &dyn Clock,
      ) -> Result<Self, WitnessError>;
  }
  ```
- **Rules**:
  - Scan with `matches_found > 0` returns `Err(WitnessError::MatchesFound)`.
  - Scan with `units_scanned == 0` without `empty_layer_declared = true` returns `Err(WitnessError::ZeroUnitsWithoutEmptyDeclaration)`.
  - `pub fn issue` marked `#[deprecated]` (prepared for visibility reduction to `pub(crate)` in W3-01).
- **ScanOutcome Caller Allowlist**:
  Only physical scanners in storage/index layers are authorized to emit `ScanOutcome`:
  - `contextra-store::lsm::memtable::MemtableScanner`
  - `contextra-store::lsm::sstable::SstableScanner`
  - `contextra-store::wal::WalScanner`
  - `contextra-vector::hnsw::HnswScanner`
  - `contextra-text::index::TextIndexScanner`
  - `contextra-graph::csr::CsrGraphScanner`

---

## 3. Footprint Hash & ProofBuilder
- **Footprint Hash**:
  `compute_witness_footprint_hash` calculates a BLAKE3 hash over the canonically sorted set of scanned units with domain prefix `b"contextra/witness-footprint/v1\0"`.
- **ProofBuilder**:
  - Added `ProofBuilder::with_required(scope, db_instance_id, deleted_keys_hash, footprint_hash, deleted_after_tx, issued_wall_ms)`.
  - Marked `ProofBuilder::new` as `#[deprecated]`.
  - Validation rejects `issued_wall_ms == 0` upon finishing.

---

## 4. Store Probe Interface (`ZeuStoreProbe`)
- **API**:
  ```rust
  pub trait ZeuStoreProbe: Send + Sync {
      #[deprecated]
      fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration;
      fn try_current_generation(&self, _layer: DeletionLayer) -> Result<u64, ProbeError>;
  }
  ```
  `WitStoreProbe` is provided as a type alias for `ZeuStoreProbe`.

---

## 5. Timestamp Anchors & Qualification Level
- **Verifier Rename**: `NoOpTsaSignatureVerifier` renamed to `AcceptAllTsaVerifier` and feature-gated behind test features (`test-utils`, `test-support`, `testing`, `test`).
- **Type-Level Qualification**:
  `QualificationLevel::Qualified(RealTsaProofToken)` requires a `RealTsaProofToken` (private constructor, constructible only by an RFC 3161 TSA verifier). Local development anchor returns `QualificationLevel::LocalMonotonic`.
- **Legal Claims**: All doc comments purged of legal/eIDAS claims.

---

## 6. Key Classes & TrustList
- **API**:
  - `KeyClass`: `Current`, `Legacy`, `Revoked`.
  - `TrustList`: Manages key IDs with validity time windows and `KeyClass`. Expired keys outside window return `KeyClass::Legacy`. Revoked keys return `KeyClass::Revoked`.
  - `ProofSignerProvider`:
    ```rust
    pub trait ProofSignerProvider: Send + Sync {
        fn signer(&self) -> Result<&dyn ProofSigner, SignerError>;
        fn db_instance_id(&self) -> Result<[u8; 16], SignerError>;
    }
    ```

---

## 7. Standalone External Verifier
- **API**:
  ```rust
  pub fn verify_standalone(
      proof: &ProofV4,
      trust_list: &TrustList,
  ) -> Result<VerifiedProof, VerifyError>;
  ```
  Operates without dependencies on `engine` or `store`. Validates canonical TLV tags, signature against `TrustList`, and checks key classes.
