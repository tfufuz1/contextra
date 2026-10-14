# Contextra — Task-Karten-Schablone (TASK_TEMPLATE.md)

> Kopiere dieses Template nach `.jules/tasks/T-2026-XXXX.toml` und fülle alle Pflichtfelder aus.
> Für Risiko-Klassen und automatische Zusatzprüfungen: `.jules/tasks/README.md`

---

## Minimal-Task-Karte (TOML)

```toml
# ── Pflichtfelder ────────────────────────────────────────────────────────────
id          = "T-2026-XXXX"
title       = "Kurzbezeichnung des Auftrags (max. 72 Zeichen)"
crate       = "contextra-<name>"            # Haupt-Crate des Auftrags
tier        = 1                             # 1 = Standard, 2 = Heavy, 0 = Doku-only
risk        = "none"                        # none|sec|crash|simd|concurrency|utf8|fusion|calib
goal        = """
Präzise Ein-Satz-Beschreibung was geändert wird und warum.
Kein Implementierungsdetail — das gehört in den Plan.
"""
non_goals   = [
  "Keine Änderungen an <X>",
  "Keine neuen Dependencies",
]

# Scope: Nur diese Dateien darf Jules ändern (AGENTS.md §6)
scope = [
  "crates/<name>/src/<datei>.rs",
  "crates/<name>/tests/<test>.rs",
]

# Protected paths: Jules darf diese NIEMALS anfassen
forbidden = [
  ".github/**",
  "xtask/**",
  "xtask-heavy/**",
  "capabilities.toml",
  "AGENTS.md",
  "rust-toolchain.toml",
  "deny.toml",
  ".jules/setup/**",
  "Cargo.lock",                             # Entfernen wenn Cargo.toml im Scope
]

# Verbindliche Invarianten aus AGENTS.md §7
invariants = ["Zero-Panic", "INV-WAL-TRUNCATION-1"]  # Beispiel

# Mindestens 1 Akzeptanzbefehl — muss grün sein vor submit
acceptance = [
  "cargo test -p <crate>",
]

# Welcher Beweis muss im PR-Body stehen
evidence_required = [
  "Neuer Failing-Before / Passing-After Test",
]

[budget]
files       = 5        # Max. Dateien die geändert werden dürfen
lines       = 200      # Max. Zeilen netto (hinzugefügte - entfernte)
iterations  = 5        # Max. check/verify Iterationen vor Stop
```

---

## Risiko-Klassen Kurzreferenz

| `risk` | Wann? | Automatische Zusatz-Gates |
|--------|-------|--------------------------|
| `none` | Reine Doku, Typen ohne Laufzeit-Effekte | – |
| `sec` | Crypto, Privacy, Egress-Audit | `mutants-diff`, `fuzz-smoke` |
| `crash` | WAL, Store, Checkpoint, Durability | `wal-replay-verify` |
| `simd` | SIMD-Distanzroutinen, Vektorisierung | `unsafe-audit` |
| `concurrency` | Neuer shared state, Lock-Hierarchie | `loom-run` |
| `utf8` | Byte-Slicing, UTF-8-Parsing | Standard |
| `fusion` | RRF, Reranking, Score-Aggregation | Standard |
| `calib` | Score-Kalibrierung, ECE, Drift | Standard |

Berührt unsafe → `touched_unsafe = true` in Karte (aktiviert `unsafe-audit`).
Berührt Cargo.toml → `touched_cargo = true` (aktiviert `dependency-gate`).

---

## Drei-Phasen-Planmuster für Jules

```markdown
## Plan: T-2026-XXXX

1. **Preflight** (Phase 1)
   - [ ] `cargo xtask repo-doctor` — Zustand prüfen
   - [ ] `cargo xtask env-attest` — Tools prüfen
   - [ ] `cargo xtask crate-context <crate>` — Crate-Kontext laden
   - [ ] `crates/<crate>/AGENTS.md` lesen — Lokale Invarianten

2. **Implementierung** (Phase 4)
   - [ ] Failing Test schreiben (rot)
   - [ ] Fix implementieren (grün)
   - [ ] Read-Back: geänderte Datei erneut lesen

3. **Verifikation** (Phase 5)
   - [ ] `cargo xtask jules check` — Scope, Lints, Symbole
   - [ ] `cargo test -p <crate>` — Akzeptanztest
   - [ ] `cargo xtask jules verify` — Gates
   - [ ] Risiko-spezifische Zusatz-Gates (gem. risk-Klasse)

4. **Submit** (Phase 7)
   - [ ] `cargo xtask jules submit` — PR-Gate + Report
```

---

## Scope-Disziplin-Checkliste

Vor der ersten Änderung:

```bash
# 1. Scope überprüfen
cargo xtask gate-guard

# 2. Symbol-Existenz prüfen (niemals Halluzinationen!)
cargo xtask symbol-exists crates/<crate>/src/<modul>::<Typ>

# 3. Typ-Duplikation vermeiden
grep -rn "struct <TypName>\|enum <TypName>" crates/ --include="*.rs"
grep "<TypName>" docs/TYPE_REGISTRY.md

# 4. API-Signatur lesen (niemals raten!)
grep -n "pub fn <methode>" crates/<crate>/src/*.rs

# 5. Parallele Branches prüfen
git branch -r | grep -i "<crate>" || echo "Keine parallelen Branches"
```
