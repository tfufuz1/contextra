# AGENTS.md — `contextra-ports`

## 1. Rolle
`contextra-ports` definiert sämtliche abstrakten Port-Schnittstellen und Trait-Definitionen zur sauberen Entkopplung von Kern und Außenwelt.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Port-Fundament)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-PRT-001:** Reine Trait-Schnittstellen (Anhang A.1, A.2) ohne konkrete Implementierungen.
- **REQ-PRT-002:** Injektionsfähige Zeit- (`Clock`) und Zufallsprimitiven (`EntropyRng`, `Rng`, `SeededRng`).
- **Durchsetzungsmechanismus (§27.6):** Statische Trait-Überprüfung, `#![forbid(unsafe_code)]`.

## 4. Verbotene Muster
- **Keine konkreten Port-Implementierungen** (außer Mocks in Tests).
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine direkten Systemzeit- oder OS-Zufallsaufrufe (`Instant::now`, `thread_rng` verboten per `clippy.toml`).

## 5. Implementierte & konsumierte Ports (Anhang A)
- **Definiert sämtliche Ports (Anhang A.1 / A.2):** `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Clock`, `EntropyRng`, `DurableLog`, `ErasureParticipant`, `KeyCustodian` etc.

## 6. Tests & Gates
- **Workspace Clippy Gate:** Verhinderung verbotener Methoden (`Instant::now`, `thread_rng`).

## 7. Offene Punkte
- keine.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Port-Katalog des Kerns ist das Modul `persistence` (`src/persistence.rs`, 352 Zeilen, Stand 22.11.2026). Es importiert nur `std`; das Crate zieht aber `contextra-types`, `blake3`, `serde`, `futures-util` und hat 26 Abhängige.
- Vertrag IF-1: Feature `persistence` (übrige Abhängigkeiten optional, SA-1); doppelte `Clock` (`clock.rs:8`) entfällt; `Cipher` von 12 auf 24 Byte Nonce; `KeyProvider` auf `ClusterId` statt `KeyDomain`; `File::write_all_at`/`truncate` (SA-3); optionale `hard_link`/`reflink` (SA-2). Fehlend: `Checksum`, `Hasher`, `Observer`, `Anchor`, `ColdTier`, `SecretAlloc`, `Weigher`, `Filter`, `ExtensionBuilder`, `Runtime`, `ClusterId`.
- Verboten: Logik im Port-Modul; Abhängigkeit des Ports vom Kern. Die Nähte `StorageEngine`, `KvBridgeStorage`, `KvPrefixStore` bleiben unverändert und sind Adapterziele ab G-5.
- Arbeitspakete: WP-1.2, WP-1.3, WP-1.6.
