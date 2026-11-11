# Befunde: `contextra-types`

**Erhebungsdatum:** 2026-11-10
**Geltungsbereich:** `crates/contextra-types`
**Referenz:** `docs/decoupling/JULES_RAHMEN.md` (R2, R9)

---

## 1. Übersicht der technischen Schulden & Befunde

### 1.1 Verwenderliste in `lib.rs` (B06)
- **Befund:** In `src/lib.rs` Zeilen 20–28 war eine veraltete Liste von 30 angeblichen Consumern hardkodiert, einschließlich des nicht im Workspace existierenden Pakets `contextra-agreement-generator`.
- **Maßnahme:** Die statische Liste wurde entfernt und durch einen normativen Verweis auf `capabilities.toml` (Vertragsebene, Ebene 2) ersetzt.

### 1.2 Veraltete Taxonomie-Begriffe in Dokumentation und Metadaten
- **Befund:** In Modul-Dokumentationen (`src/`), `README.md`, `AGENTS.md` und `capabilities.fragment.toml` wurden veraltete Bezeichnungen (wie Ring 0, Ring 1, Klasse B, Welle 6) verwendet.
- **Maßnahme:** Bereinigt auf die neutrale Architektur der Vertragsebene (Ebene 2, Gruppe `contracts`). `capabilities.fragment.toml` wurde auf Schema v3 aktualisiert.

### 1.3 Löschdomänen-Typen (`erasure.rs`, `witness.rs`, `ids.rs`)
- **Befund:** In `erasure.rs`, `witness.rs` (`ScanMethod`, `SagaRecord`) und `ids.rs` (`ErasureId`, `GateId`) befinden sich Löschtypen, die als `#[deprecated]` markiert sind und fachlich zur Erasure-Familie (`contextra-erasure`) gehören.
- **Maßnahme:** Standardisierte Migrationskommentare (`// MIGRATION(S-70): ...`) hinzugefügt. Die Typen bleiben für Abwärtskompatibilität erhalten (INV-2).

### 1.4 Diagnose: Dead-Code-Probe
- **Befund:** `tools/dead_code_microprobe.py` meldete 33 angebliche ungenutzte private Items. Eine Detailanalyse zeigte:
  - 17 davon sind Proptest-Funktionen (`prop_*`), die über Makros expandiert und aufgerufen werden.
  - 9 davon sind Serde-Hilfsfunktionen (`default_*`), die über `#[serde(default = "...")]` referenziert werden.
  - 7 sind Test-Hilfsfunktionen.
  - Keine echten toten Produktionsfunktionen vorhanden.

### 1.5 Diagnose: Isolations-Tool (`tools/isolation_proof.py`)
- **Befund:** `tools/isolation_proof.py` erzeugt im temporären Workspace ein leeres `[workspace.dependencies]`-Manifest. Dadurch scheitert das Tool für jedes Crate, das Dependencies via `{ workspace = true }` erbt (z. B. `ahash`, `serde`).
- **Maßnahme:** Als Handoff an Z in `REPORT.md` dokumentiert. Die tatsächliche Isolation wurde durch Analyse der Abhängigkeits-Closure und Standalone-Kompilierung nachgewiesen.
