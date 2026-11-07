# AGENTS.md — `contextra-erasure`

> **Ring 3** · Klasse `[K]` (Baustein) · Reifegrad `beta` · Unsafe: nein · Rolle: Eigenständiger Löschkern: Saga, Vault, ReadGate/GateLog, Journal, Ledger, Messung/Witness, signierte Berichte, RestoreGuard.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-erasure --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `crypto`, `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `crypto` |
| Features | keine (`default = []`) |
| Lints | `#![forbid(unsafe_code)]`; **keine `mod.rs`** (Wurzeldatei `<modul>.rs` neben `<modul>/`) |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll. **Kein Konsument im Workspace** (kein `Cargo.toml` hängt davon ab): Verdrahtung durch `engine`/Fassade fehlt (WP 3.5/3.8/3.9).
- Dev-Abhängigkeiten: keine; Tests nutzen `test_mocks.rs` und eigene Fakes.
- **Darf nie** `store`, `vector`, `text`, `graph` importieren; Kommunikation nur über `Purger`/`LayerScanner`/`StorageBackend`/`ErasureTarget` aus `ports`.
- Wird `crypto::deletion_proof`/`ed25519_proof`/`kv_shredding`/`revocation_log`/`engine::contextra_impl::erasure` übernehmen (Phase 3).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `saga/` (`core`, `driver`, `machine`, `seams`, `idempotency`, `retention`) | `ErasureCore`, Zustandsmaschine `Requested→Gated→KeyDestroyed→Purging→Measured→Completed` (bzw. `Restricted`) |
| `vault/` (`table`, `slot`, `durable`, `keyring`, `sealer`, `rehash`, `scrub`, `open`) | `FileVault`: 16 Shards, Slots, Zeroing, Poisoning |
| `gate/` (`read_gate`, `erased_set`, `log`, `batch`) | RCU-Lesesperre, `ErasedSet`, `GateLog` |
| `journal/`, `ledger/` | Crash-sicheres Saga-Journal, append-only Ledger |
| `purge/` (`window`, `order`, `reserve`, `cx`) | Orchestrierung der `Purger` |
| `measure/` (`scanner_flow`, `positive_control`, `witness`, `field`) | Positivkontrolle, Messung, `LayerWitness` |
| `report/` (`build`, `canonical`, `risk`, `signing`) | kanonische, signierte Berichte |
| `restore/` (`guard`, `replay`) | `RestoreGuard` |
| `scope/`, `backend.rs`, `io.rs`, `replace.rs`, `restricted.rs`, `supersede.rs`, `error.rs` | Scope-Codec, Seams, I/O, Sonderfälle, `ErasureError` |

## 4. Verträge & Invarianten

1. **Messen vor Behaupten (A3):** kein Bericht ohne Scanner-Witness; Klasse nie höher als die Messung trägt (INV-CLASS-MEASURED-1). Kein Pfad erzeugt einen Beweis ohne Witness (INV-NO-DUMMY-PROOF-1).
2. **Reihenfolge (INV-DEL-ORDER-1):** Schlüsselvernichtung erst nach Bereinigung und durablem Inventar (Ausnahme: Notpfad).
3. **Keine Wiederauferstehung (INV-NO-RESURRECTION-1):** nach Start und Restore kein lebender Schlüssel einer gelöschten Einheit.
4. **Vault-Poisoning:** jeder I/O-/`fsync`-Fehler beim Überschreiben/Nullen vergiftet den Handle unwiderruflich (fail-closed).
5. **Lesesperre zuerst:** `ReadGate` ist armiert, bevor ein Lesepfad Verkehr annimmt (INV-BOOT-1); `GateLog` überlebt Neustarts.
6. **Saga idempotent und crash-sicher:** Wiederanlauf aus dem Journal; Abbruch nur zwischen Schritten (INV-CANCEL-1).
7. **Signierschlüssel** nur aus KMS/KeyManager, nie als frei übergebener Byte-String (INV-PROOF-KEY-1); Verifikation mit falschem Schlüssel schlägt fehl.
8. **Wording (INV-CLAIM-BIND-1):** Berichtstexte nur aus fester Satzbibliothek, je Satz an einen Messwert gebunden; Annahmen/Ausschlüsse (Flash, RAM, Swap, externe Backups) stehen im Bericht. Nie „garantiert“, „100 %“, „DSGVO-konform“.

## 5. Ports

**Konsumiert:** `Purger`, `LayerScanner`, `ReadGate`, `LedgerSink`, `ReportSigner`, `RecordSealer`, `FileKeyring`, `Vfs`, `Clock`, `Rng`, `IdGen`. **Implementiert:** `ReadGate`-/`LedgerSink`-Standard (prüfen am Code).

## 6. Nebenläufigkeit & Performance

- Pro Einheit läuft genau eine Saga (Idempotenzschlüssel). Gate-Updates linearisierbar (`gate_linearization_test.rs`, `loom_gate.rs`).
- Kein globaler Zustand; `tokio`/`futures-util` nur an der Saga-Schale (Heuristik: 12 `async fn`); Kern der Zustandsmaschine rein synchron halten.
- Benchmark: `benches/bm_erase.rs`.

## 7. Anti-Patterns

- Konkrete Speicher-Crates importieren.
- `mod.rs` anlegen.
- Bericht oder Witness von Hand konstruieren (Konstruktoren sind `pub(crate)`).
- Klasse aus Konfiguration statt aus Messung ableiten.
- Absolute Rechts-/Garantiesprache.

## 8. Tests & Messung

```bash
cargo test -p contextra-erasure --locked
cargo clippy -p contextra-erasure --all-targets -- -D warnings
RUSTFLAGS="--cfg loom" cargo test -p contextra-erasure --test loom_gate
cargo bench -p contextra-erasure
```

- Vorhanden: Crash-Matrix (`crash_matrix_recovery.rs`), Spec-Tests je Modul, echte Scans (`measure_real_scan_tests.rs`), Vault-Konformität.
- **Sabotage-/Mutationstests** (SPEC §17.11): jeder Fehler in Bereinigung, Vernichtung, Messung oder Berichtsbildung muss mindestens einen Test brechen.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.1 | übernimmt `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs` aus `crypto` |
| 3.5 | übernimmt `engine/contextra_impl/erasure.rs` |
| 3.7/3.8 | MCP-Löschwerkzeuge und Fassaden-API als optionale Brücke |
| 3.9 | Finalisierung, `example-erasure` (WP 6.1) |
| 2.2 | Port-Signaturen angleichen (Root-§10 K2/K3) |
