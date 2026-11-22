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

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: liefert die produktiven Port-Implementierungen `Cipher`, `Mac`, `Kdf`, `Hasher`, `FileKeyProvider` für den Kern.
- Ist (22.11.2026): AES-256-GCM-SIV mit 12-Byte-Nonce, HKDF, BLAKE3 und weitere Primitive; **keine** Implementierung eines `persistence`-Ports; `KeyManager` implementiert `KeyProvider` nicht.
- Vertrag IF-3: `Cipher` mit 24-Byte-Nonce (Vorschlag XChaCha20-Poly1305, UE-10, Review nach Spec §1.5), `Mac`/`Kdf`/`Hasher` nach Spec §4, `FileKeyProvider` mit zufälligen Clusterschlüsseln (D-19). Schlüssel stehen nie im Datenverzeichnis des Kerns.
- Verboten: der Kern hängt nie an diesem Crate (Allow-Liste Spec §3.5); keine Panik, kein Schlüsselmaterial in `Debug`/Logs.
- Arbeitspakete: WP-2.4. Nachweis: `cipher_suite`, `mac_suite`, `kdf_suite`, `key_provider_suite`.
