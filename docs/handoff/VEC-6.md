# Handoff VEC-6 — contextra-vector: Entkopplung & Härtung

**Datum:** 10. November 2026
**Commit-Kennung:** VC6
**Branch:** `jules/vec-6-decouple`
**Status:** Abbruch am START-GATE (Blockiert)

---

## 1. START-GATE Befund

Gemäß Auftragsvorgabe:
```bash
START-GATE (alle Befehle müssen erfolgreich sein, sonst Abbruch)
    test -d crates/contextra-ids && test -d crates/contextra-env
    grep -q 'contextra_ids' crates/contextra-types/src/lib.rs
    grep -q 'contextra_env' crates/contextra-ports/src/lib.rs
    cargo check -p contextra-vector --all-targets   (Ergebnis in die Baseline)
Schlägt eines der ersten drei fehl, ist das Fundament-Paket P0 nicht gemergt: Brich ab, schreibe nur docs/handoff/VEC-6.md mit dem Blocker und ändere keinen Code.
```

### Ausführung der Prüfbefehle:
1. `test -d crates/contextra-ids && test -d crates/contextra-env`: **Erfolgreich** (Verzeichnisse existieren).
2. `grep -q 'contextra_ids' crates/contextra-types/src/lib.rs`: **FEHLGESCHLAGEN** (Exit Code 1).
   - In `crates/contextra-types/src/lib.rs` findet sich kein Bezug auf `contextra_ids` (die Re-Exports liegen aktuell in Submodulen wie `src/ids.rs`, `src/refs.rs`, `src/tenant_scope.rs`, `src/types/domain/ids.rs`).
3. `grep -q 'contextra_env' crates/contextra-ports/src/lib.rs`: **FEHLGESCHLAGEN** (Exit Code 1).
   - In `crates/contextra-ports/src/lib.rs` findet sich kein Bezug auf `contextra_env` (die Re-Exports liegen in `src/clock.rs`, `src/rng.rs`, `src/id_gen.rs`, `src/env.rs`, `src/metrics.rs`).

Da zwei der ersten drei Prüfungen fehlgeschlagen sind, greift die bindende Vorgabe:
**"Schlägt eines der ersten drei fehl, ist das Fundament-Paket P0 nicht gemergt: Brich ab, schreibe nur docs/handoff/VEC-6.md mit dem Blocker und ändere keinen Code."**

---

## 2. Status & Nächste Schritte

- **Status:** Vorhaben abgebrochen / blockiert am START-GATE.
- **Code-Änderungen:** Keine Code-Änderungen an `crates/contextra-vector/**` oder anderen Crates vorgenommen.
- **Koordinationsanforderung:** Entweder die START-GATE-Definition anpassen (z. B. Prüfung auf `crates/contextra-types/src/ids.rs` und `crates/contextra-ports/src/env.rs` statt `src/lib.rs`) oder in P0 / Fundament die Re-Exports direkt in `crates/contextra-types/src/lib.rs` und `crates/contextra-ports/src/lib.rs` verankern.
