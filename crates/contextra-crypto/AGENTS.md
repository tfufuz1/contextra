# AGENTS.md — `contextra-crypto`

## 1. Rolle
`contextra-crypto` stellt I/O-freie kryptographische Primitiven, Schlüsselverwaltung, Zeroization, Audit-Chains und DeletionProof-Erzeugung/Verifikation bereit.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Kryptographie & Nachweislogik, I/O-frei, lock-frei in Produktionspfaden)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Persistenz nur über den Port `DurableLog`; Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §20)
- **REQ-CRY-001:** Kanonische TLV-Kodierung (`TlvEncoder`, `TlvField`) mit Big-Endian Tag (`u16`) / Length (`u32`) Sortierung.
- **REQ-CRY-002:** Ed25519 strict Signaturprüfung (`verify_strict`) und BLAKE3 Hashketten.
- **REQ-CRY-003:** Disclaimers & Claims gemäß Taxonomie (§20.12): `AUDIT_SCOPE_DISCLAIMER` verpflichtend in Claim-Texten; Vertraulichkeits- & Abwesenheitsbeweise ausgeschlossen.
- **REQ-CRY-004:** Two-Phase Fail-Closed KMS Revocation (`LocalKmsProvider::destroy_key`).
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, offline `verify_proof` Tests, Testvektoren.

## 4. Verbotene Muster
- **Kein `unsafe` außerhalb der dedizierten Inseln** (`#![forbid(unsafe_code)]` strikt erzwungen).
- **Kein I/O:** Persistenz erfolgt ausschließlich über den Port `DurableLog`.
- **Keine Löschtypen außerhalb `crypto`/`ports`** (§6.10, Punkt 5).
- Keine Erzeugung unverifizierter Proofs bei fehlenden Kernschichten (`possesses_full_core_coverage`).
- Keinerlei Verwendung verbotener Werbe-Claims ("100% gelöscht", "garantiert", "DSGVO-konform").

## 5. Implementierte & konsumierte Ports (Anhang A)
- Konsumiert: Port `DurableLog` für Persistenz, `EntropyRng` für CSPRNG Nonces.
- Implementiert: `KmsProvider`, DeletionProof Offline-Verifier.

## 6. Tests & Gates
- **Fuzzing (REQ-QA-003):** WAL-Replay & DeletionProof Deserialisierungs-Fuzzer.
- **Multithread Concurrency:** Thread-Safety Tests für Vaults und Nonces (`concurrency_vault_and_nonce.rs`).
- **KMS Revocation Gate:** `kms_revoke_fail_closed.rs`.

## 7. Offene Punkte
- **P-01 bis P-14 (§20.13):** Offene juristische Prüfpunkte bezüglich Schutzbedarf und Schlüsselvernichtungs-Attestierung.
- **F-01 bis F-08 (§34):** Architekturentscheidungen zu Crypto-Erase und DeletionProof v4.
