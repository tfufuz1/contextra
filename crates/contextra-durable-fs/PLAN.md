# contextra-durable-fs — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Crash-sicheres Schreiben: `tmp → fsync → rename → fsync(Verzeichnis)`, Gruppen-Sync, Dateisperre (SPEC 15, 3.1). Grundlage für Durability (INV-FSYNC-1). Abhängigkeiten nach Anhang E: `env`, `ids`, `sys`. Ziel-Umfang: 2.500 Zeilen. Ist: laut Zielspalte 3.491 Zeilen (Altbestand).

## 2. Vertrag
Öffentlich: `vfs` (Dateiabstraktion), `fault_injector`, `journal`, `scrub`, `health`, `platform`. SPEC 15 verlangt Atomares Schreiben und Gruppen-Sync. Die Absturzmatrix nutzt `FaultVfs` aus `contextra-testkit` (SPEC 15).
Ist-Abweichung: `fault_injector.rs` gehört nach REM-13 entfernt (Fault-Injection raus, SPEC 3.1).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `env`, `ids`, `sys`.
**Ist (Cargo.toml):** `thiserror`, `serde` (optional), `tracing` (optional), `crc32fast` (optional, Feature `crc`). Keine internen Abhängigkeiten im Ist außer über die Liste (gegen `cargo metadata` zu prüfen, siehe Abnahme).
**Features:** `default = ["std", "crc"]` (R3-Ausnahme, SPEC 3.2). Ist-Abweichung: `inspect` und `tracing`-Feature sind in SPEC 15 nicht genannt.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/vfs.rs` | Dateisystem-Abstraktion | Soll (SPEC 15) |
| `src/platform.rs` | plattformabhängige Operationen | Soll |
| `src/journal.rs` | Journal | zu prüfen |
| `src/scrub.rs` | Scrub-Hilfen | zu prüfen, Zuordnung nach SPEC 15 offen |
| `src/health.rs` | Gesundheitsstatus | zu prüfen; `Poisoned` ist nach SPEC 3.1 zu entfernen |
| `src/fault_injector.rs` | Fault-Injection | **REM-13: entfernen** |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-07 | in SPEC 15 genannt, Gegenstand in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| REM-13 | `fault_injector.rs`, Fault-Injection und `Poisoned` | `grep -rn "FaultInjector\|Poisoned" crates/contextra-durable-fs/src` leer |
| REM-14 | ANCHOR-Dateien (7), `contract.rs` | `grep -rl "ANCHOR\[" crates/contextra-durable-fs` leer |

## 6. Umbau
1. `fault_injector.rs` und `Poisoned` entfernen (REM-13). Testhilfen gehen nach `contextra-testkit`.
2. Atomares Schreiben und Dateisperre gegen die Absturzmatrix prüfen.
3. Selbstdeklarationen entfernen (nach Freigabe).

**Offene Punkte (Entscheidung nötig):**
- **O-1** REM-07: Inhalt in SPEC 12 nachlesen und bestätigen.
- **O-2** Features `inspect` und `tracing`: aufnehmen oder entfernen?
- **O-3** Zuordnung von `journal.rs`, `scrub.rs`, `health.rs` (in SPEC 15 nicht genannt).

## 7. Tests
- Absturzmatrix mit `FaultVfs` aus `contextra-testkit` (SPEC 15).
- Ist-Befund: nicht gezählt, bei Bedarf nachmessen.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | kein `unsafe`, `forbid(unsafe_code)` in `lib.rs` | eingehalten |
| G-06 | `default = ["std", "crc"]` (R3-Ausnahme) | erlaubt |
| G-09 | 7 Dateien mit ANCHOR/FILE-CONTEXT | verletzt (REM-14) |
| G-12 | 1 Treffer `Instant::now`/`SystemTime::now` außerhalb `env` | verletzt, Fundstelle prüfen und auf `Clock` umstellen |
