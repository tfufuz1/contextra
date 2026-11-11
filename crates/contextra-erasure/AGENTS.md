# AGENTS.md — `contextra-erasure`

> **Ebene 0 (compliance)** · Klasse `A` (Baustein) · Reifegrad `stable` · Unsafe: nein (`#![forbid(unsafe_code)]`) · Rolle: Eigenständiger, entkoppelter Löschkern: Saga, Vault, ReadGate/GateLog, Journal, Ledger, Messung/Witness, signierte Berichte, RestoreGuard.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`. Stand 2026-11-11, verifiziert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo check -p contextra-erasure --no-default-features` |
| Contextra-Abhängigkeiten **Ist** | 0 interne Abhängigkeiten (vollständig autark) |
| Features | `default = ["signing", "vault", "async"]` |
| Lints | `#![forbid(unsafe_code)]`; **keine `mod.rs`** (Wurzeldatei `<modul>.rs` neben `<modul>/`) |
| Zero-Panic | Erzwungen für Produktion via Clippy Lints |

## 2. Architekturprinzipien & Entkopplung

- **Vollkommene Autarkie:** Keine Abhängigkeiten zu `contextra-types`, `contextra-ports` oder `contextra-crypto`.
- **Eigene Domänentypen:** Alle benötigten Schnittstellen (`Purger`, `LayerScanner`, `StorageBackend`, `ErasureTarget`, `Clock`, `Rng`) sind autonom in `src/types_local.rs` definiert.
- **Kryptografische Isolation:** Krypto-Operationen (HKDF, AEAD, Ed25519) sind modular gekapselt.
- **Feature-Gating:**
  - `signing`: Aktiviert Ed25519-Signierung und externe Beweisprüfer (`ed25519-dalek`).
  - `vault`: Aktiviert das verschlüsselte 16-Shard FileVault-Subsystem (`aes-gcm-siv`).
  - `async`: Aktiviert Tokio-basierte asynchrone Treiber und Seams (`tokio`, `futures-util`).

## 3. Modul-Karte

| Pfad | Verantwortung |
|---|---|
| `saga/` (`core`, `driver`, `machine`, `seams`, `idempotency`, `retention`) | `ErasureCore`, Zustandsmaschine `Requested→Gated→KeyDestroyed→Purging→Measured→Completed` |
| `vault/` (`table`, `slot`, `durable`, `keyring`, `sealer`, `rehash`, `scrub`, `open`) | `FileVault`: 16 Shards, Slots, Zeroing, Handle-Poisoning |
| `gate/` (`read_gate`, `erased_set`, `log`, `batch`) | RCU-Lesesperre, `ErasedSet`, `GateLog` |
| `journal/`, `ledger/` | Crash-sicheres Saga-Journal, append-only Ledger |
| `purge/` (`window`, `order`, `reserve`, `cx`) | Sequenzielle Bereinigung und Purge-Orchestrierung |
| `measure/` (`scanner_flow`, `positive_control`, `witness`, `field`) | Positivkontrolle, Messung, `LayerWitness` |
| `report/` (`build`, `canonical`, `risk`, `signing`) | Kanonische, deterministische Löschberichte |
| `restore/` (`guard`, `replay`) | Schutz vor Reaktivierung gelöschter Schlüssel |
| `kv_shredding.rs`, `revocation_log.rs` | Schlüsselhierarchie (Group-KEK, Record-DEK) und Widerrufslog |
| `types_local.rs` | Autarke Typ- und Portdefinitionen für das gesamte Crate |

## 4. Verträge & Invarianten

1. **INV-CLASS-MEASURED-1:** Kein Bericht ohne Scanner-Witness; Nachweisstufe entspricht genau der tatsächlichen Messung.
2. **INV-DEL-ORDER-1:** Schlüsselzerstörung erfolgt erst nach Bereinigung und durablem Inventar (außer im Notpfad).
3. **INV-NO-RESURRECTION-1:** Nach Systemstart und Wiederanlauf existiert kein aktiver Schlüssel für widerrufene Einheiten.
4. **INV-VAULT-POISONING:** Jeder E/A- oder Synchronisationsfehler beim Nullungs-Vorgang vergiftet das Handle permanent (Fail-Closed).
5. **INV-BOOT-1:** Das `ReadGate` sperrt Lesezugriffe bereits vor Aktivierung des Datenverkehrs.
6. **INV-CANCEL-1:** Die Saga ist idempotent und bricht nur an definierten Zustandsgrenzen sauber ab.
7. **INV-WORDING-1:** Neutrale Dokumentation und Berichterstattung ohne Rechtsgarantien (SPEC §6.4).

## 5. Kommentar-Syntax (Comment-Anchors)

Verbindliche Systematik für Entwickler und Agenten:
- `// SECTION <Name>` / `// !SECTION` für zusammenhängende logische Einheiten.
- `// ANCHOR[contextra-erasure:<BEREICH>-<NNN>]` als persistente Referenzpunkte.
- `// NOTE:`, `// TODO:`, `// FIXME:`, `// STUB:` für Inline-Hinweise.
- `// LINK <Pfad>#<Anker>` für Querverweise zwischen Modulen.
