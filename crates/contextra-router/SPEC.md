# contextra-router — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v3, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse und Test-Suite-Lauf am geklonten Repo-Stand (`main`, HEAD `5408826f8`).
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md v3 übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-router/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Adaptive Query- und Modell-Routing-Engine: Arm-Registry, Conformal Calibration (Gibbs & Candès), Multi-Armed Bandits (LinUCB, Thompson Sampling), Kaskaden-Auswahl und Dispatch. *[S]*

## 2. Zielzustand (docs/SPEC.md v3 §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | llm |
| Ebene (Kern-Closure) | E2 (Integration) |
| Kern-Closure | types, ports, adapt (`wire` nur hinter Feature `subprocess-dispatch`) |
| Default-Features | leer |
| Fehlertyp (R3) | besteht (`ArmRegistryError`, `FcTsDispatchError`, etc.) |
| Brücke (hinter Feature `workspace`, §1.1) | — (E2-Integrationsschicht) |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1a/E1b = Bausteine · E2 = Integration (`router`, `types`, `ports`, `engine`, `db`, `agent`, `cognition`, `contextra`) · E3 = Anwendungen und Testwerkzeuge.

## 3. Ist-Zustand (tools/ & Cargo)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 2324 Zeilen in 16 Dateien (Gesamt inkl. Tests ~5838 LOC) |
| Version / publish | workspace-vererbt / unset |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | `contextra-types`, `contextra-ports`, `contextra-adapt`, `contextra-wire` (optional hinter `subprocess-dispatch`) |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist) | `subprocess-dispatch`, `bandit-routing`, `cloud-egress-guard`, `egress-sherman-morrison`, `flow-corrected-thompson`, `exp-fc-ts`, `exp-routing-strategy` |
| Default-Features (Ist) | keine (leer) |
| zero-panic Befunde (`zero_panic_analyzer.py`) | 0 (keine unberechtigten Panics in Produktion) |
| unsafe: Vorkommen | 0 (`#![forbid(unsafe_code)]` strikt enforced) |
| Determinismus (`determinism_guard.py`) | 0 Befunde (deterministische Tie-Breaks und RNG-Port-Verwendung) |
| Modularitäts-Score (`decouple_inspector.py`) | 61/100 |

## 4. Ist-vs-Ziel-Abgleich [V]

- **Interne Abhängigkeiten:** `types`, `ports`, `adapt` als Soll-Integrationsbestandteile für E2; `wire` optional hinter `subprocess-dispatch` [V].
- **Default-Features:** Ist = Ziel (leer).
- **Vfs-Injektion:** Vollständig umgesetzt in `lifecycle.rs` via `new_with_vfs`, `try_new_with_vfs` und `read_calibration_bytes`.
- **Rückwärtskompatibilität:** `ports_local.rs` bleibt vorerst als reiner Re-Export-Pfad erhalten, bis Downstream-Konsumenten in Folgeschritten migriert sind.

## 5. Anforderungen & Regeln (docs/SPEC.md v3 §1.2, §3.2) [S]

- **INV-1 Formatgleichheit:** Keine Schema-Änderung ohne Versionierung.
- **INV-2 Pfadgleichheit:** Re-Exports bei Typ-Verschiebungen beibehalten.
- **INV-3/INV-4:** Kompilierbarkeit und Testbarkeit bleiben nach jedem Schritt grün.
- **INV-5 Determinismus:** Deterministische Modellauswahl, Tie-Break nach Profilname/ID aufsteigend.
- **R1 Richtung:** E2 hängt nur an E0, E1 und anderen E2 ab, nie an E3.
- **R10 Feature-Regel:** `default = []` bleibt strikt eingehalten.

## 6. Abnahme [S]

```bash
cargo check -p contextra-router --no-default-features --locked
cargo test -p contextra-router --all-features --locked
cargo clippy -p contextra-router --all-targets --all-features -- -D warnings
python3 tools/decouple_inspector.py -c contextra-router
python3 tools/zero_panic_analyzer.py --crate contextra-router
python3 tools/determinism_guard.py --crate contextra-router
```
