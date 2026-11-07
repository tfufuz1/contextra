# AGENTS.md — `contextra-crypto`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-crypto` stellt reine, I/O-freie kryptographische Primitiven, Schlüsselableitung (HKDF), authentifizierte Chiffren (AES-256-GCM-SIV), Audit-Chains, Zeroization und KMS-Provider bereit (Ring 0).
Gemäß der Baukasten-Architektur wurden sämtliche rechtlichen Lösch- und Nachweiskomponenten (Multi-Level-Deletion-Proofs, Saga-Verdrahtung, DSGVO-Begrifflichkeiten) vollständig nach `contextra-erasure` ausgelagert.
`contextra-crypto` fungiert als neutraler, mathematisch rigoroser Krypto-Kern, der ohne Seiteneffekte in Fremdprojekten verwendet werden kann.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Kryptographie & Schlüsselverwaltung, I/O-frei, lock-frei im Heißpfad)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **REQ-CRY-001:** Kanonische TLV-Kodierung (`TlvEncoder`, `TlvField`) mit Big-Endian Tag (`u16`) / Length (`u32`) Sortierung.
- **REQ-CRY-002:** Ed25519 strict Signaturprüfung (`verify_strict`) und BLAKE3 Hashketten.
- **REQ-CRY-004:** Two-Phase Fail-Closed KMS Revocation (`LocalKmsProvider::destroy_key`).
- **AAD-Bindung (ADR-N27):** Scope-granulare Schlüsselverwaltung (`ScopeKeyring`) mit expliziter AAD-Bindung zur Verhinderung von Block-Swapping.
- **Zeroize-on-Drop:** Sämtliches Schlüsselmaterial wird bei Freigabe deterministisch genullt (`zeroize::Zeroizing`, `DekHandle`).
- **Wording-Compliance & Neutralität:** Strenges Verbot von unwissenschaftlichen Marketing- oder Rechtsclaims ("100% gelöscht", "absolut DSGVO-konform").

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]` strikt erzwungen).
- **Kein Dateisystem- oder Netzwerk-I/O** (Persistenz nur über Ports wie `DurableLog`).
- Keine Multi-Level Deletion-Saga- oder DSGVO-Logik im Crate (vollständig in `contextra-erasure`).
- Keine globalen statischen Schlüssel-Caches.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `KmsProvider`, `FileKeyring`, `RecordSealer`.
- **Konsumiert:** `EntropyRng` aus `contextra-ports` für Nonces, `Clock` für Zeitstempel.

## 6. Tests, Sabotage & Qualitätssicherung
- **Testvektoren & Fuzzing (REQ-QA-003):** Standardisierte RFC-Vektoren, WAL-Crypto Fuzzing und TLV-Deserialisierungs-Fuzzer.
- **Concurrency & Thread-Safety:** Parallele Thread-Safety Tests für Vaults und Nonces (`concurrency_vault_and_nonce.rs`).
- **KMS Revocation Gate:** `kms_revoke_fail_closed.rs` zur Verifikation des Fail-Closed Verhaltens.

## 7. Offene Punkte & Technische Schulden
- DEBT-01: Vollständige Bereinigung verbliebener Übergangs-Reexporte in Abstimmung mit `contextra-erasure`.
