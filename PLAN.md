# Implementierungsplan: Workspace (Wurzel)

**Geltungsbereich:** `Cargo.toml`, `Cargo.lock`, `capabilities.toml`, `deny.toml`, `.cargo/config.toml`, `scripts/`, `tools/`, `tests/` (Root), `CLAUDE.md`, `docs/`.
**Basis:** SPEC R14 §2.1 (14 Crates), §2.7, §12, §13.1 (Phasen und Gates), §13.3, DEC-52, DEC-53; R15 §2.2 und §5.

## 1. Ist-Befund (gemessen)

- `[workspace].members` nennt **15** Einträge, davon `crates/contextra-kvcache`, dessen Verzeichnis fehlt. Jeder `cargo`-Aufruf scheitert. SPEC: 14.
- `Cargo.lock` enthält `contextra-kvcache` (1 Treffer).
- `[workspace.dependencies]` enthält Altpfad-Abhängigkeiten: `bincode`, `flatbuffers`, `memmap2`, `tokio` (full), `parking_lot`, `crossbeam-skiplist`, `regex`, `roaring`, `criterion`, `ort`/`tokenizers`/`hf-hub`/`ndarray` (auskommentiert, Frozen).
- `[workspace.lints]` entspricht §2.6 (deny für `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `await_holding_lock` u. a.).
- `[profile.release]` mit `lto = "fat"`, `codegen-units = 1`, `overflow-checks = true`. Bei langen Builds auf 1 vCPU kritisch (CLAUDE.md §3). Nicht geändert, aber CI-Laufzeit prüfen.
- `scripts/check_forbidden_apis.sh`: greppt `crates/contextra-storage-ports/src` (existiert nicht), `|| true` verschluckt den Fehler, Ergebnis ist **immer** „OK“ (R15 B-14).
- Architektur-Tests liegen in `tests/` im Root und werden von keinem Crate eingebunden (CLAUDE.md §4). Verzeichnis `tests/` existiert im Klon nicht; die Dateien `layering.rs` und `unsafe_islands.rs` sind nur als Verweise in `CLAUDE.md` belegt.
- `capabilities.toml`: beschreibt Crates, die nicht existieren; 7 Mitglieder fehlen, `contextra-vault` steht dort, ist aber kein Mitglied (CLAUDE.md §4).
- `tools/reference/`: `generator.py` und `verify_goldens.py` existieren. Anhang C der SPEC sagt „nicht vorhanden“ (FIX-40, überholt).
- `tools/ctx.sh gates`: prüft Konsistenz von `capabilities.toml`, Ergebnis nicht gelesen.
- Git-HEAD `ce4579a56`, 6080 Commits.

## 2. Entfernen

| Ziel | Datei | Aktion |
|---|---|---|
| Mitglied `crates/contextra-kvcache` | `Cargo.toml` | streichen (P0-A, §13.2 Nr. 1) |
| Eintrag `contextra-kvcache` | `Cargo.lock` | über `cargo update` nicht erlaubt (`--locked`); Lock neu erzeugen mit Commit-Notiz, da ein Build die Datei sonst nicht ändert |
| Alias `xtask` | `.cargo/config.toml` | entfernen (zeigt auf nicht existierendes Paket) |
| `bincode`, `flatbuffers`, `memmap2`, `parking_lot`, `crossbeam-skiplist` als Workspace-Default | `Cargo.toml` `[workspace.dependencies]` | nur Crate-lokal deklarieren; Verbote in `deny.toml` (R14 §13.2 Nr. 10) |
| `tokio` mit Feature `full` als Default | `Cargo.toml` | nur in `engine` mit `rt`, `sync`, `macros` als benötigt |
| Frozen-Zone-Kommentare für `ort`, `tokenizers`, `hf-hub`, `ndarray`, `reqwest`, `openraft`, `tonic`, `prost` | `Cargo.toml` | in `docs/agent/FROZEN.md` verschieben, aus `Cargo.toml` entfernen |
| Phantom-Eintrag `contextra-storage-ports` | `scripts/check_forbidden_apis.sh` | entfernen |

## 3. Ändern

- **`Cargo.toml`:**
  - `members` auf 14 Einträge (die Liste aus §2.1 SPEC, Reihenfolge nach Ebenen).
  - `[workspace.package].rust-version = "1.89"` bleibt; prüfen, ob `rust-toolchain.toml` identisch ist.
- **`capabilities.toml`:** aus SPEC §2.1 neu erzeugen. Format: `name`, `layer`, `group`, `tier`, `deps`. Genau 14 Einträge. `contextra-vault` entfernen, falls kein Crate mit diesem Namen geplant ist.
- **`CLAUDE.md`:**
  - Abschnitt 4 (Architektur): auf 14 Crates korrigieren.
  - Abschnitt 5 (Bekannte Befunde): mit Verweis auf R15 aktualisieren.
  - Abschnitt 7 (Agenten): Regel unverändert.
  - Verweise auf fehlende `tests/`-Dateien korrigieren, sobald Tests verschoben sind.
- **`deny.toml`:** Verbotsliste je Ebene als `wrappers`-Regel (R14 §13.2 Nr. 10):
  - L2-Crates ohne `tokio`, `bincode`, `blake3`, `memmap2`, `scc`, `dashmap`, `parking_lot`, `aes-gcm-siv`.
  - L0/L1 ohne `serde`, `futures` (außer `ports::retrieval`).
  - `ignore`-Einträge nur mit Begründung und Ablaufdatum (SPEC §2.7).
- **`scripts/check_forbidden_apis.sh`:**
  - Pfade aus `members` ableiten, nicht hart codieren.
  - Fehlender Pfad ⇒ `exit 1` (kein `|| true` auf dem Pfad-Check).
  - Selbsttest: ein Fixture mit `blake3::` in `crates/contextra-types/src/__selftest.rs` muss scheitern; Fixture danach löschen.
  - Muster erweitern um `tokio::`, `bincode::`, `memmap2`, `block_on`, `aes_gcm_siv`, `parking_lot`, `crc32fast`, `scc::`, `dashmap`, `static [A-Z_]*KEY` (DEC-53).
- **`scripts/check_invariant_refs.sh`, `canary_scan.sh`:** nicht gelesen; vor Umbau prüfen.
- **Root-`tests/`:**
  - `layering.rs` und `unsafe_islands.rs` nach `crates/contextra-engine/tests/architecture.rs` verschieben (DEC-52).
  - Im Root kein `tests/`-Verzeichnis mehr.
- **`tools/ctx.sh gates`:** Konsistenzprüfung gegen `capabilities.toml` und `members` ausbauen; Ausgabe als Fehlerliste, Exit-Code ≠ 0 bei Abweichung.

## 4. Implementieren

1. **Gate-Skript `tools/gates.sh`** (oder Erweiterung von `ctx.sh`):
   - `cargo metadata --locked --format-version 1` muss fehlerfrei sein.
   - Jedes Mitglied hat Verzeichnis, `Cargo.toml`, `src/lib.rs`.
   - `capabilities.toml` und `members` stimmen überein.
   - Verbotsskript ohne Treffer in den vorgesehenen Ebenen.
   - `unsafe_islands`: nur `contextra-sys` und `contextra-simd` ohne `forbid`.
2. **CI-Matrix:**
   - Stufe 1: `cargo check -p <crate> --locked` je Crate, parallel, auf 1 vCPU sequenziell (Hardware-Beschränkung CLAUDE.md §3).
   - Stufe 2: `cargo test -p <crate> --locked` je Crate.
   - Stufe 3: `cargo test --workspace --locked` erst, wenn Stufen 1 und 2 grün sind.
   - Stufe 4: `cargo clippy` mit Zählung der Warnungen als Messbasis (R14 §13.2 Nr. 14).
   - Stufe 5: `cargo deny check`.
3. **Crash-Sweep-Registrierung:** Liste aller `Fs`-Operationen, append-only (SPEC §13.1).
4. **Golden-Vektoren:** `tools/reference/generator.py` mit `verify_goldens.py` als Pflicht-Gate für G-F (Anhang C).
5. **Messbericht-Template** für §16 (Hardware, Dateisystem, Last, Methode, Streuung).

## 5. Tests

- `gates.sh` selbst testen: absichtlich verletzte Bedingung muss scheitern (je Gate einen Negativtest).
- `check_forbidden_apis.sh` mit Selbsttest (siehe §3).
- `cargo metadata` in CI nach jeder Änderung an `Cargo.toml`.

## 6. Abhängigkeiten

Keine Crate-Abhängigkeiten auf Workspace-Ebene. `[workspace.dependencies]` dient nur der Versionsfixierung; nach Umbau nur noch Abhängigkeiten, die mindestens zwei Crates nutzen.

## 7. Gate und Reihenfolge

1. `kvcache` aus `members` streichen, `Cargo.lock` neu erzeugen (Commit-Notiz). **Erster Schritt, ohne ihn baut nichts.**
2. Verbotsskript reparieren und Selbsttest (DEC-53).
3. `capabilities.toml` aus §2.1 neu erzeugen.
4. `deny.toml` mit Ebenenregeln.
5. Architektur-Tests verschieben (DEC-52).
6. CI-Matrix Stufe 1 und 2.
7. **G-0a:** jedes der 14 Crates baut einzeln, `cargo metadata` fehlerfrei, Verbotsskript hat Selbsttest.

**Risiken:**
- `Cargo.lock` lässt sich mit `--locked` nicht durch normale Builds ändern. Die Neuerzeugung ist ein bewusster Schritt, kein Nebeneffekt.
- 1 vCPU und 3,9 GB RAM: `lto = "fat"` im Release-Profil macht Builds sehr lang. Für Entwicklung `profile.dev` prüfen; Release nur in CI.
- Sechstausend Commits im Verlauf: `git log` für Herkunft einzelner Altdateien nutzen, nicht für Entscheidungen.
