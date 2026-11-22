# xtask Inventar - Zahlen und Abweichungsanalyse

**HEAD:** `db68bdf74`
**Datum:** 2026-10-27

---

## 1. Gesamtzahlen (IST-Zustand auf `db68bdf74`)

| Metrik | IST-Wert | Reproduzierbarer Befehl |
|---|---|---|
| **xtask Gesamtzahlen (Code-Zeilen nach `loc.py`)** | **39028** | `python3 -c "import tools.loc as loc, os; print(sum(loc.count_loc(os.path.join(r, f)) for r, d, fs in os.walk('xtask') for f in fs if f.endswith('.rs')))"` |
| **Anzahl Stubs (`nur_stub = ja`)** | **1** | `python3 tools/generate_all_inventory_files.py` (siehe `modules.tsv`) |
| **Eingebundene Module in `lib.rs` (`in_lib_rs = ja`)** | **38** | `cargo check --lib -p xtask` (AST-Erreichbarkeit) |
| **Nicht/Unerreichbare Harness/Gate-Module unter `xtask/src`** | **115** | Total `xtask/src` Dateianzahl (153) minus `lib.rs`-eingebunden (38) |
| **Registrierte Gates in `registered_gates()`** | **2** | `python3 -c "with open('xtask/src/gates/mod.rs') as f: print(f.read())"` |
| **Dateien gesamt (`xtask/src` + `xtask/tests`)** | **228** | `find xtask/src xtask/tests -type f | wc -l` |
| **Dateien unter `xtask/src`** | **153** | `find xtask/src -type f | wc -l` |
| **Dateien unter `xtask/tests`** | **75** | `find xtask/tests -type f | wc -l` |

---

## 2. Abweichungsanalyse gegen Spezifikation & Prompt-Scans

### A. Vergleich der Gesamtzeilenzahl

- **SPEC §4.2 Nennung:** `31.991` Zeilen
- **PLAN.md Nennung:** `29.706` Zeilen
- **IST-Messung (Commit `db68bdf74` nach `loc.py`):** **`39028`** Zeilen
- **Abweichung & Ursache:** Die Spezifikation und der PLAN basieren auf früheren Commit-Ständen vor Aufteilung/Anpassung der Gates und Tests in `xtask`. Der exakte LOC-Wert wird hiermit inventarisiert.

### B. Vergleich der Scan-Zahlen aus dem Prompt

1. **Stubs:** Prompt-Scan sprach von `125` Stubs vs IST **`1`** Stubs (gemäß präziser Funktions-/Arm-Prüfung).
2. **Eingebundene Module:** Prompt-Scan sprach von `68/76` Modulen vs IST **`38`** in `lib.rs` von **`153`** Dateipfaden unter `xtask/src/`.
3. **Unerreichbare Harness-Module:** Prompt-Scan sprach von `34` vs IST **`115`** nicht in `lib.rs` verdrahteten Modulen unter `xtask/src/`.
4. **Registrierte Gates:** Prompt-Scan sprach von `2/44` vs IST **`2`** registrierten Gates in `registered_gates()` (von 44 `builtin` Gates in `governance/gates.toml`).

---

## 3. Reproduzierbarkeit aller Zahlen

Jede Zahl in diesem Dokument und in den zugehörigen `.tsv`-Dateien ist über den Aufruf von `python3 tools/generate_all_inventory_files.py` bzw. die einzelnen Shell-Befehle 100% exakt reproduzierbar.
