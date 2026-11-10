# Pending API Changes (Fragment P01)

Format: Symbol | betroffene Crates | gewünschte Änderung

`deletion_proof_active` in `DurabilityMode::validate_against_features` | `contextra-store`, `contextra-ports` | Veraltetes Deletion-Proof-Flag in Validierungssignaturen vollständig deprecaten / abbauen (Bereits `validate_durability` vorhanden)
