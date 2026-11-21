# contextra-env — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch)
**Stand:** 21.11.2026, Abgleich mit SPEC v4.1

Verweist nur auf Kapitel und IDs der SPEC. Abweichungen des Ist-Zustands sind als **Ist-Abweichung** markiert, ungeklärte Punkte stehen unter **Offene Punkte (Entscheidung nötig)** in Abschnitt 6.

---

## 1. Zweck und Ebene

`contextra-env` stellt die injizierbare Umgebung bereit: `Clock`, `SeededRng` und `MetricsSink` (SPEC 3.1, 15). Es ist die **einzige Stelle**, an der `Instant::now` und `SystemTime::now` aufgerufen werden dürfen (R10, G-12).

Ebene E0, Abhängigkeit nur `contextra-ids` (SPEC 3.2, 3.3, Anhang E). Ziel-Umfang nach SPEC 3.1: 2.000 Zeilen. Ist: 2.437 Zeilen `src` (laut `CLAUDE.md`, 21.11.2026).

## 2. Vertrag

Soll nach SPEC 15: `Clock` (real, `ManualClock`), `SeededRng`, `MetricsSink`.

| Soll | Ist | Status |
|---|---|---|
| `Clock` (real) | `pub trait Clock`, `SystemClock`, Funktion `unix_ms` (`clock.rs`) | vorhanden |
| `ManualClock` | `manual_clock.rs` | vorhanden |
| `SeededRng` | `pub struct SeededRng` (`rng.rs`), Trait `Rng` | vorhanden; Trait-Name `Rng` ist im Soll nicht genannt |
| `MetricsSink` | `pub trait MetricsSink`, `NoopMetricsSink`, `MetricKey`, `HistogramSnapshot` (`metrics.rs`) | vorhanden |
| `IdGen`, `SequentialIdGen` | `id_gen.rs` | nicht in SPEC 15 genannt, offen (siehe 6) |
| `SequenceLog`, `SeqLogEntry` | `seq_log.rs` (872 Zeilen) | nicht in SPEC 15 genannt, offen (siehe 6) |

Features: Soll keine (SPEC 15). Ist `std`, `test-utils`. Siehe 3.

## 3. Abhängigkeiten

**Erlaubt (Anhang E):** `contextra-ids`.

**Cargo-Auszug (Ist):**
```toml
[dependencies]
contextra-ids = { workspace = true }
tracing = { workspace = true }
```

Ist-Abweichung: `tracing` ist in SPEC 15 nicht genannt. Entscheidung nötig (siehe 6).

**Features:** `default = []` eingehalten (R3). Ist-Abweichung: `std = []` und `test-utils = []` stehen im Soll nicht. Verbleib offen.

**Dev-Abhängigkeiten:** keine im Abschnitt, aber `[[test]]`-Einträge (`legacy_reference_unix_ms`) vorhanden.

## 4. Module

| Datei | Verantwortung (Ist, Modul-Doku) | Befund |
|---|---|---|
| `src/lib.rs` | Kanonische Umgebungsabstraktionen, Re-Exporte | ok |
| `src/clock.rs` | `Clock`-Trait und Systemuhr | Soll |
| `src/manual_clock.rs` | steuerbare Uhr für Tests | Soll |
| `src/rng.rs` | `Rng`-Trait, deterministisches `SeededRng` (SplitMix64) | Soll |
| `src/metrics.rs` | `MetricsSink` und In-Memory-Implementierung | Soll |
| `src/id_gen.rs` | `IdGen`, `SequentialIdGen` | offen (6) |
| `src/seq_log.rs` | `SequenceLog` für snapshot-isolierte `_at`-Suchen | offen (6) |
| `src/env.rs` | Modul-Doku Umgebung, Re-Exporte | ok |
| `tests/conformance_env.rs` | Konformität der Traits | ok |
| `tests/golden_rng.rs` | Golden-Vektoren für `SeededRng` (FND.1, INV-1) | ok |
| `tests/legacy_reference_unix_ms.rs` | Differenztest gegen eingefrorene Referenz | offen (7) |

## 5. Entfernungen

Im SPEC-Bestand von Kapitel 12 sind für `env` keine REM-Punkte benannt. Entfernt werden:

| Gegenstand | Begründung | Abnahme-Befehl |
|---|---|---|
| Selbstdeklarationen (`ANCHOR[`, `FILE-CONTEXT`) in 10 Dateien | REM-14 | `grep -rl "ANCHOR\[\|FILE-CONTEXT" crates/contextra-env` leer |
| `unwrap`/`expect`/`panic!`-Treffer in `src` (11), soweit nicht in `#[cfg(test)]` | R6, G-13 | `grep -rn "unwrap()\|expect(\|panic!(" crates/contextra-env/src` leer außerhalb von Tests |

## 6. Umbau

1. Selbstdeklarationen entfernen. Diese Aufgabe ist nach CLAUDE.md §6 Punkt 9 erst nach Freigabe des Nutzers zulässig.
2. `unwrap`/`expect`/`panic!` in Produktcode prüfen und abbauen (R6), Ratchet nicht erhöhen.
3. Testmodule und Test-Hilfen klar von Produktcode trennen (`test-utils`-Feature, siehe O-2).

**Offene Punkte (Entscheidung nötig):**
- **O-1 `IdGen` und `SequenceLog`.** SPEC 15 nennt sie nicht. Gehören sie zu `env`, zu `ids` oder zu einem Besitzer (`version` für `_at`-Suchen, SPEC 4.5, 6.6)? Entscheidung des Nutzers.
- **O-2 Features `std` und `test-utils`.** SPEC 15 nennt keine Features. Entfernen oder aufnehmen?
- **O-3 Abhängigkeit `tracing`.** Nicht in SPEC 15. Behalten oder entfernen?
- **O-4 Trait-Name `Rng`.** SPEC 15 nennt nur `SeededRng`. Umbenennen oder Namen in SPEC aufnehmen?

## 7. Tests

- Clock-Semantik, Zähler ohne Nutzdaten (T-LOG-01, SPEC 15).
- Golden-Vektoren für `SeededRng` (`tests/golden_rng.rs`).
- Konformität (`tests/conformance_env.rs`).
- Ist-Befund: 54 `#[test]` (laut `CLAUDE.md`).
- Der Differenztest `tests/legacy_reference_unix_ms.rs` verweist auf eingefrorene Referenzen; Verbleib offen (O-1 bzw. Abschnitt 6).

## 8. Abnahme

| Gate | Ist-Befund | Soll |
|---|---|---|
| G-01 | Build laut `CLAUDE.md` ok | `cargo check`, `test`, `clippy -D warnings` grün |
| G-02 | `#![forbid(unsafe_code)]` in `lib.rs`, kein `unsafe` | eingehalten |
| G-03 | Abhängigkeit `tracing` nicht in Anhang E | offen (O-3) |
| G-06 | `default = []` | eingehalten |
| G-09 | 10 Dateien mit `ANCHOR`/`FILE-CONTEXT`, keine `contract.rs` | verletzt (REM-14), Abbau nach Freigabe |
| G-11 | keine `compat`-Verzeichnisse | eingehalten |
| G-12 | Zeit nur hier erlaubt: 3 Treffer `Instant::now`/`SystemTime::now` in `contextra-env` | eingehalten (Ausnahme dieses Crates) |
| G-13 | 11 Treffer `unwrap`/`expect`/`panic!` in `src` | zu prüfen, Anteil in `cfg(test)` |
