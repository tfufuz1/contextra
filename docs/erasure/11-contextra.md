# Implementierungsspezifikation `contextra` (Facade) (Löschkette – Builder, Profil)

*Ring 4 · Konzept-WP 5 (Teil), 12 · Abhängig von: 10 · Blockiert: 12 · Index: `00-INDEX.md` (K-8, K-42, A-2)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

Die Facade stellt die Erase-API **ohne** neue `pub fn` bereit (Invariante ≤ 20 `pub fn`, ✔ `lib.rs:3`), verdrahtet Profil → `ErasureConfig`/`StoreErasureConfig`, setzt die Schlüsselwurzel (A-2) und korrigiert die irreführende Doku zu `deletion_proof_active` (K-42).

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `contextra-types`, `-ports`, `-db`, `tokio`, `tracing` (`Cargo.toml`); `[lints] workspace` |
| Exporte | `lib.rs:46-57`: `AgentMemory`, `ContextraBuilder`, `ContextraConfigExt`, `EncryptionMode`, `CollectionProfile`, `DeploymentTier{EdgeMinimal, PowerUserLocal, EnterpriseShared, EnterpriseRegulated}`, `KvDeleteMode`, `LsmTuning`, aus `contextra_db`: `Collection`, `Contextra`, `ContextraConfig`, …; `PerformanceProfile`, `DurabilityMode`; `LlmQueryRewriter` |
| Funktionen | Drei `pub fn`/`pub async fn` in `lib.rs`: `builder(dimension)` (`:85`), `open(path)` (`:127`), `open_with_config(…)` (`:159`) |
| Builder | `ContextraBuilder{storage_path, config: ContextraConfig, encryption_mode, encryption_error, explicit_user_config, embedder, performance_profile}` (`builder.rs:83`); Abgleich `config.deletion_proof_active` ↔ `resolved.integrity_proof_active` (`builder.rs:330-340`) |
| `deletion_proof_active` | steuert die **Durability-Prüfung** (`WalNoHmac` unzulässig bei `integrity_proof_active`, `performance_profile.rs:72-85`), **nicht** einen Löschnachweis (K-42). `lifecycle.rs:21`: `open` setzt `deletion_proof_active: false` |
| Profile | `DeploymentTier::resolve() -> CollectionProfile` (`collection_profile.rs:146-`), `EnterpriseRegulated` verbietet Auto-Extraktion (Test `regulated_auto_extraction_default.rs`) |

## 3. Änderungen

| Datei | Aktion |
| --- | --- |
| `src/lib.rs` | Re-Exporte: `pub use contextra_types::{EraseTarget, ErasureId, ErasureReason, ErasurePolicy, ErasureProfile, ErasureStatus, ErasureState, Guarantee, ErasureException, AwaitLevel, CheckpointTaintPolicy, ReinsertPolicy}` und `pub use contextra_db::{EraseOptions, EraseHandle, ErasePlanSummary, ErasureConfig}` (falls `contextra-db` re-exportiert; sonst Pfad **[?]** `rg "pub use" crates/contextra-db/src/lib.rs`). **Keine neue `pub fn`** (K-8): `erase`, `erase_status`, `erase_dry_run` hängen als `impl Contextra` in der Engine (10 §12) |
| `src/builder.rs` | Methoden am **Builder-Typ** (zählen nicht zur Modul-`pub fn`-Invariante **[?]** Invariantentest prüft Datei `lib.rs`; Zählweise vor Umsetzung lesen): `erasure_config(ErasureConfig)`, `erasure_root_path(PathBuf)`, `external_ledger(PathBuf)`, `erasure_profile(ErasureProfile)` |
| `src/collection_profile.rs` | `CollectionProfile.erasure: ErasureProfile` (Standard für alle außer `EnterpriseRegulated`) |
| `src/performance_profile.rs`, `lifecycle.rs` (Engine) | Doku korrigieren (K-42): „`deletion_proof_active` bedeutet **Integritätsnachweis aktiv** und steuert nur die Durability-Prüfung; es gibt keinen Löschnachweis in diesem Pfad. Der Löschnachweis ist das Erase-Ledger (03 §11).“ Feldname bleibt (API-Stabilität) |
| `tests/deletion_e2e.rs` | Canary `T-DEL-1` (13 §7), Legacy-Symbole entfernt (05 §3.1) |
| `tests/collection_profile_matrix.rs` | Legacy-Symbole entfernen; Matrix um `erasure` erweitern |
| `tests/erasure_facade_invariants.rs` | **NEU** (§6) |

## 4. Builder und Profil (normativ)

```rust
impl ContextraBuilder {
    pub fn erasure_config(mut self, c: ErasureConfig) -> Self;
    /// Verzeichnis für `root.key` (A-2: FileErasureRoot). Default: <storage_path>/erasure.
    pub fn erasure_root_path(mut self, p: impl Into<PathBuf>) -> Self;
    /// Ledger-Pfad **außerhalb** des Datenverzeichnisses (03 §11.1). Pflicht im Profil Regulated.
    pub fn external_ledger(mut self, p: impl Into<PathBuf>) -> Self;
    pub fn erasure_profile(mut self, p: ErasureProfile) -> Self;
}
```

Auflösung beim `build()`/`open`:

| Profil | `ErasureProfile` | Wurzel (A-2) | Ledger | `require_external_ledger` | weitere |
| --- | --- | --- | --- | --- | --- |
| `EdgeMinimal`, `PowerUserLocal`, `EnterpriseShared` | `Standard` | `FileErasureRoot` (`<erasure_dir>/root.key`, 0600), oder `DerivedErasureRoot`, wenn `EncryptionMode::Passphrase` gesetzt | optional | false | `auto_erase_on_delete = false` |
| `EnterpriseRegulated` | `Regulated` | **Externe** Wurzel Pflicht (`DerivedErasureRoot` aus Passphrase/KMS **oder** `external ErasureRootProvider` mit `is_external() == true`) | Pflicht, außerhalb `data_dir` | true | `auto_erase_on_delete` konfigurierbar (ADR-F, Default aus), `checkpoint_policy = Drop`, Karenz 0 |

- Regulated + `FileErasureRoot` (nicht extern) ⇒ `build()` liefert `Err(ContextraError::invalid_input("regulated profile requires external erasure root"))` **vor** dem Öffnen des Stores (A-2: Start verweigert). Test TST-FAC-ERA-3.
- Regulated ohne `external_ledger` ⇒ gleiche Behandlung.
- Der Builder gibt die Wurzel als `Arc<dyn ErasureRootProvider>` an `LsmConfig`/`LsmStorage::open` weiter (Schnittstelle in 03 WP-3.x); er reicht **nie** Schlüsselbytes durch Logs (`Debug` von `ContextraBuilder` zeigt nur Pfade, ✔ `builder.rs:93-107` erweitern um `erasure_root_path`, nicht um Material).
- `ContextraConfigExt` (bestehend) wird um `erasure_config()`/`set_erasure_config()` ergänzt (**keine** `pub fn` in `lib.rs`).

## 5. API-Fläche und Dokumentation

- Doku an `Contextra::erase`/`erase_status`/`erase_dry_run` (Engine) mit „# Fehler“, „# Crash“ und **Garantie-Tabelle** (G0/G1/G3/G2).
- Modul-Doku der Facade: Abschnitt „Löschen“: `delete` = Retire (G0, zeitreisefähig), `erase` = G1+G3 mit Nachweis; externe Kopien (Backups/Exporte/Replikate/Client-Caches/LLM-Anbieter) sind **ausdrücklich** außerhalb (`OutOfScopeExternalCopies`).
- `AgentMemory::forget` (`agent_memory.rs`) bleibt Retire; ein zusätzlicher `AgentMemory::erase(&MemoryId)` ist **optional** (WP-11.4) und ruft `erase(Document)`.

## 6. Tests

| ID | Inhalt |
| --- | --- |
| TST-FAC-ERA-1 | `rg -c "pub (async )?fn" crates/contextra/src/lib.rs` ≤ 20 (bestehender Invariantentest bleibt grün) |
| TST-FAC-ERA-2 | `erase`/`erase_status`/`erase_dry_run` über die Facade-Typen aufrufbar (`use contextra::*`) |
| TST-FAC-ERA-3 | Regulated ohne externe Wurzel/ohne externes Ledger ⇒ `build()` `Err`; Standard ⇒ ok, `root.key` existiert, Modus 0600 (Unix) |
| TST-FAC-ERA-4 | Neustart zwischen zwei Öffnungen: `erasure_id_hmac` stabil, Ledger `verify` besteht (Regression K-33) |
| TST-FAC-ERA-5 | `open`-Pfad (`lifecycle.rs:21`) startet mit Standard-Profil, Erase funktioniert |
| TST-FAC-ERA-6 | Debug-Ausgabe von Builder/Config enthält keine Schlüsselbytes |
| TST-FAC-ERA-7 | Doku-Test: Text zu `deletion_proof_active` entspricht K-42 (Doctest/`rg`) |
| TST-FAC-ERA-8 | `tests/wiring_matrix.rs` um Erase-Verdrahtung ergänzt: jedes Profil registriert Text, Vektor, Graph, Checkpoints |

## 7. Arbeitspakete

| WP | Inhalt |
| --- | --- |
| 11.1 | Re-Exporte, Doku-Korrektur K-42 |
| 11.2 | Builder-Methoden, Profil-Auflösung, Wurzel/Ledger-Verdrahtung |
| 11.3 | Tests, `deletion_e2e.rs` auf `T-DEL-1` |
| 11.4 | (optional) `AgentMemory::erase` |

## 8. Definition of Done

Invariantentest grün; Regulated-Verweigerung getestet; `deletion_e2e.rs` ohne Legacy-Symbole; keine neue Kante in `capabilities.toml` (Facade bleibt bei `db`/`ports`/`types`, K-36).

## 9. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-F1 | Zählt die Invariante nur `lib.rs`? | Lesen des Invariantentests; Builder-Methoden bleiben am Typ |
| O-F2 | Re-Export-Pfad über `contextra-db` | prüfen, sonst `contextra-engine`-Typen nicht direkt importieren (Facade kennt nur `db`) |
| O-F3 | Passphrase-Ableitung der Wurzel | 05 O-C2 |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Verbindliche Ergänzungen.** (1) **Startreihenfolge** 14 §3.11. Der Builder registriert `TextErasure`, `GraphErasure`, `HnswErasure`, `CheckpointErasure` (Registry-Order Text → Graph → Vektor → Checkpoint) **nach** Kernel-Öffnung und **vor** Freigabe der Lesepfade; vor `mask_layers` liefern Lesepfade `NotReady`. (2) **Profile:** *Regulated* ⇒ `required_layers = {Text, Vector, Graph, Checkpoints}` (soweit gebaut), externe Wurzel und externes Ledger Pflicht, `StaticKeyMaterialSource` verboten; *Standard*/*Embedded* ⇒ Löschung läuft, `Sealed` weist fehlende Layer als Ausnahme aus (`purged = false`). (3) `deletion_proof_active` steuert weiterhin nur die Durability-Prüfung (K-42); Doku und Rustdoc benennen das. (4) Die Facade-Invariante ≤ 20 `pub fn` bleibt: `erase*` liegt als `impl Contextra` in der Engine (K-8). (5) Test `TST-ENG-WIRE-1` (14 §6).
