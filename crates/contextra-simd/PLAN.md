# contextra-simd — Implementierungsspezifikation

**Ebene:** E0 (Fundament, `unsafe`-Insel, G-02)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Distanzkerne L2, Dot und Cosine mit SIMD-Kernen und Skalar-Referenz, Dispatch (SPEC 15, CMP-7, REM-23). Zweite erlaubte `unsafe`-Insel neben `contextra-sys`. Ziel-Umfang: 3.500 Zeilen (SPEC 3.1). Ist: laut Zielspalte 6.461 Zeilen (Altbestand; Messung im Ist nicht erneuert).

## 2. Vertrag
Öffentlich: Distanzfunktionen mit Dispatch über Skalar und SIMD (AVX2, AVX512, NEON laut Ist-Dateien). Varianten liefern identische Ergebnisse wie die Skalar-Referenz (Property-Test, SPEC 15).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** keine.
**Ist-Abweichung:** `contextra-types` (optional, Feature `workspace`). Das verletzt R1 und R7 (REM-27, CMP-7). Entfernen.
**Features:** Ist `default = ["std"]`, `std`, `serde`, `workspace`, `types-compat`, `simd-avx512`. R3 erlaubt `default = ["std"]` ausdrücklich (SPEC 3.2). Ist-Abweichung: `workspace` und `types-compat` verletzen R7/CMP-10 und sind zu entfernen. `serde` und `simd-avx512` sind in SPEC 15 nicht genannt.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/kernels/scalar.rs` | Skalar-Referenz | Soll |
| `src/kernels/avx2.rs`, `avx512.rs`, `neon.rs` | SIMD-Kerne | Soll (Zielarchitekturen zu bestätigen) |
| `src/kernels/mod.rs`, `src/dispatch.rs` | Auswahl zur Laufzeit | Soll |
| `src/metric.rs` | Metriken | Soll |
| `src/compat/mod.rs` | Kompatibilitätsmodul | **REM CMP-7, R7**: aufzulösen, Verzeichnis entfernen (G-11) |
| `src/contract.rs` | Selbstdeklaration | REM-14 |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| CMP-7 / R7 | `src/compat/` | `find crates/contextra-simd -type d -name compat` leer |
| REM-27 | Abhängigkeit `contextra-types` | `grep -rn "contextra_types\|contextra-types" crates/contextra-simd` leer |
| REM-23 | Gegenstand laut SPEC 12 (in SPEC 15 genannt) | Inhalt in SPEC 12 nachlesen, offen |
| REM-14 | `contract.rs`, ANCHOR-Dateien (19) | `grep -rl "ANCHOR\[" crates/contextra-simd` leer |

## 6. Umbau
1. `compat/` auflösen: Inhalte in `kernels/` und `dispatch.rs` verschieben (CMP-7).
2. Feature `workspace` und `types-compat` entfernen (CMP-10).
3. Abhängigkeit `contextra-types` lösen (REM-27): benötigte Typen lokal oder über `ids`.
4. Selbstdeklarationen entfernen (nach Freigabe).

**Offene Punkte (Entscheidung nötig):**
- **O-1** Feature `serde` und `simd-avx512`: aufnehmen oder entfernen? SPEC 15 nennt keine Features.
- **O-2** REM-23: Die SPEC nennt die Entfernung ohne Liste. Welche Varianten gehören dazu?
- **O-3** Unwrap-Treffer (4) in `src`: prüfen, ob in Tests.

## 7. Tests
- Property-Test: SIMD gegen Skalar (SPEC 15).
- Ist-Befund: nicht gezählt. Bei Bedarf mit `grep -rc "#\[test\]"` nachmessen.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | `unsafe` in 4 Dateien, kein `forbid` (erlaubte Insel) | erlaubt |
| G-07 | `contextra-types` als optionale Abhängigkeit | verletzt (REM-27) |
| G-09 | 19 Dateien mit ANCHOR/FILE-CONTEXT | verletzt (REM-14) |
| G-11 | `compat/`-Verzeichnis, Features `workspace`, `types-compat` | verletzt (CMP-7, CMP-10) |
| G-13 | 4 Treffer | Abbau nach R6 |
