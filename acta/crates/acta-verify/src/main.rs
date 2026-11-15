//! acta-verify: Unabhaengige Pruefung von Berichten und Ledgern; keine interne Abhaengigkeit.
//!
//! Level L9 · Budget 1200 LOC · Spezifikation docs/SPEC.md Kap. 17
//!
//! Extraktionsarchiv: `archive/acta-verify/MANIFEST.md` (falls vorhanden).

fn main() {
    // Spec 17: Stufen V1 bis V3. Keine interne Abhaengigkeit.
    println!("acta-verify {}", env!("CARGO_PKG_VERSION"));
}
