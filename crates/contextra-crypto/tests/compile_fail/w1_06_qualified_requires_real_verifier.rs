// UI compile-fail test verifying that QualificationLevel::Qualified cannot be constructed without
// a RealTsaProofToken issued by a real verifier.

use contextra_crypto::QualificationLevel;

fn main() {
    let _ = QualificationLevel::Qualified;
}
