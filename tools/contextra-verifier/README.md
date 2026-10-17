# Contextra DeletionProof Independent Verifier (Python)

Independent, language-agnostic verifier for **Contextra DeletionProof Version 4 (TLV)** proofs (§C7, ADR-13, INV-D8).

## Overview

This standalone Python tool verifies Contextra DeletionProof v4 binary TLV and JSON proof artifacts without any dependency on the Rust codebase or runtime. It serves as an independent third-party auditor tool to attest cryptographic signatures, canonical TLV encoding, and policy compliance.

## Prerequisites

- Python 3.10 or higher
- Dependencies specified in `requirements.txt`:
  - `blake3`
  - `cryptography`

Install dependencies:

```bash
pip install -r requirements.txt
```

## Command Line Usage

To verify a proof file (`proof.bin` or `proof.json`) against an Ed25519 public key (32 bytes binary or 64-char hex string):

```bash
python -m contextra_verifier.cli path/to/proof.bin --pubkey path/to/pubkey.bin
```

Structured JSON output mode:

```bash
python -m contextra_verifier.cli path/to/proof.bin --pubkey path/to/pubkey.bin --json
```

### Exit Codes

| Exit Code | Category | Description |
|-----------|----------|-------------|
| `0` | `VALID` | Proof signature and all policy rules verified successfully. |
| `1` | `INVALID` | Signature mismatch, payload tampering, or key ID mismatch. |
| `2` | `FORMAT_ERROR` / `POLICY_VIOLATION` | Malformed TLV, unknown critical tag, or policy invariant failure. |
| `3` | `USAGE_ERROR` | Missing key, unreadable input file, or invalid CLI parameters. |

## Verification Scope & Limitations (Garantieumfang)

### Was der Verifier BELEGT

1. **Kanonische Format- & Kodierungsintegrität**:
   - Gültiges TLV-Format mit streng monoton aufsteigender Tag-Sortierung.
   - Ablehnung unbekannter kritischer Tags (`0x8000`-Bit gesetzt) außerhalb des bekannten V4-Schema-Bereichs.
2. **Kryptografische Authentizität & Signaturgültigkeit**:
   - Gültige Ed25519-Signatur über `contextra/deletion-proof/v4\0` || `canonical_tlv`.
   - Kryptografische Bindung des Unterzeichners: `signer_key_id` (Tag `0x8014`) entspricht exakt `BLAKE3(pubkey)`.
3. **Policy- & Schema-Konsistenz**:
   - **Instanzbindung (`db_instance_id`, Tag `0x8003`)**: Nicht-leerer Instanzidentifikator.
   - **Stufenkonsistenz (`stage`, Tag `0x8004`)**: Stufe 1 (Logical) oder Stufe 2 (Physical).
   - **Kettenbruchschutz (`supersedes`, Tag `0x8005`)**: Keine Selbstreferenz auf `proof_id` und keine Null-Bytes.
   - **Geschlossene ClaimText-Enumeration (INV-D7, Tag `0x800C`)**: Claim-IDs strikt innerhalb des geschlossenen Katalogs `{1..8}` (S0..S4, RamUnreferenced, RamZeroized).
   - **Assurance-Level**: Stufen S0 bis S4 (`0..4`).
   - **Pflicht-Exclusions (Rule R5, Tag `0x800D`)**: Vollständige Abdeckung aller gesetzlich/spezifikationsgemäß geforderten Ausschlussbereiche (Standard: IDs 1..9; bei S4 Crypto-Erase: IDs 1, 2, 4..9).
   - **Umgebungsblock-Tri-States (Tag `0x800F`)**: Gültige Drei-Zustands-Werte (`0`: Unknown, `1`: Yes, `2`: No).
   - **Zeitliche Schranken**: Scan-Zeitpunkt `scanned_at_tx` >= `deleted_after_tx`.
   - **Konsistenz gelöschter Schlüssel**: `deleted_keys_hash == 0` erfordert `observed_empty == True`.

---

### Was der Verifier NICHT BELEGT (Explizite Nicht-Garantie)

> **IMPORTANT / WICHTIGER HINWEIS**:
> Der Verifier erbringt **KEINEN physikalischen oder messtechnischen Beweis** für die tatsächliche Abwesenheit von Daten auf Speichermedien (z. B. SSD-Remanenz, Flash Wear-Leveling, Hardware-Caches, Backup-Medien außerhalb des Prozesses).
>
> - Der Verifier prüft ausschließlich die **kryptografische Authentizität, Vollständigkeit und Policy-Konsistenz** des vorliegenden Nachweis-Artefakts.
> - Er attestiert, dass die austellende Engine zum Nachweiszeitpunkt signierte Zeugnisse und Scans nach Spezifikation V4 erzeugt hat.
> - Die physische Ausführung der Löschprozesse (SSTable Purge, WAL Zeroization, Key Destruction) obliegt der ausführenden Engine-Runtime.

## Running Tests & Test Vector Generator

To run the complete verifier unit test suite across all 25 test vectors:

```bash
python -m pytest tests
```

To re-generate test vectors v01–v25 reproducibly using deterministic test keys:

```bash
python tools_dev/generate_test_vectors.py
```
