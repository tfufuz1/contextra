# Governance Rule: WAL & Cryptographic Invariants (`rules/wal_crypto.md`)

## Gilt für
- `crates/contextra-store`
- `crates/contextra-crypto`
- Alle Storage- und Crypto-Interaktionen in Contextra Cognitive OS

## Pflichtregeln
1. **WAL-First Consistency (P1/P28)**: Jede krypto-relevante oder persistente Mutation muss vor ihrer Anwendung im In-Memory-Index dauerhaft im Write-Ahead Log (`crates/contextra-store/src/wal/`) protokolliert werden (Quelle: `crates/contextra-store/AGENTS.md`, ADR-005).
2. **HMAC-Integrationsschutz (INV-WAL-LEGACY-KEY-1)**: WAL-Segmente nutzen HMAC-SHA256 mit rotierbarem Master-Key. Der Zugriff auf den obfuscated Legacy-Schlüssel ist strikt auf MIGRATION-Pfade über `Wal::open_for_legacy_migration` beschränkt (Quelle: `crates/contextra-store/src/wal/reader.rs`).
3. **Zeroize für Cryptographic Keys**: Alle Speicherstrukturen, die Schlüsselmaterial verwalten, müssen `zeroize::Zeroize` oder `ZeroizeOnDrop` implementieren, um Key-Remanenz im RAM zu verhindern (Quelle: `crates/contextra-crypto/src/ed25519_proof.rs`, ADR-012).
4. **Asymmetrische Deletion Proofs (AK-20/AK-21)**: Löschnachweise müssen extern verifizierbar sein (Ed25519 Signaturen via `DeletionProof::verify`) ohne Abhängigkeit von Datenbank-Zustand oder KeyManager (Quelle: `crates/contextra-crypto/src/deletion_proof.rs`, `docs/reports/deletion_proof_verifiability_report.md`).

## Häufige Fehler
- Nutzung unverschlüsselter Puffer für sensible Keys im RAM.
- Umgehung des WAL-Protokolls bei Batch-Writes im LSM-Tree.
- Verwenden von symmetrischem Secret-State für Dritte bei Deletion Proof Verification.

## Verweise
- `crates/contextra-store/src/wal/`
- `crates/contextra-crypto/src/deletion_proof.rs`
- `docs/decisions/`
