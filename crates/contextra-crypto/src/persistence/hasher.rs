//! `Hasher` als unkeyed BLAKE3 (SPEC §4, D-12; §16.7 und §26: Standardbelegung).
//!
//! Der Port `Hasher` fehlt im Port-Stand auf `main`. Deshalb steht die Methode inhärent,
//! bis P1 den Port angleicht. Die Kette ist unkeyed; sie erkennt Teilmanipulation nur
//! zusammen mit dem Anker (SPEC D-12).

/// Unkeyed BLAKE3 mit 256-Bit-Ausgabe. Zustandslos.
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Hasher;

impl Blake3Hasher {
    /// Hasht die Teile hintereinander und liefert 32 Byte.
    pub fn hash256(&self, parts: &[&[u8]]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        for part in parts {
            hasher.update(part);
        }
        *hasher.finalize().as_bytes()
    }
}
