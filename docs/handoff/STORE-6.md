# Handoff STORE-6 — contextra-store: StoreError, compat/-Brücke, Durability-Nachweise

**Datum:** 10. November 2026
**Commit-Kennung:** ST6
**Branch:** `jules/store-6-decouple`
**Status:** ABGEBROCHEN (Blocker: START-GATE fehlgeschlagen)

---

## 1. Blocker / START-GATE-Befund

Gemäß verbindlicher Arbeitsanweisung:
```text
START-GATE (alle Befehle müssen erfolgreich sein, sonst Abbruch)
    test -d crates/contextra-ids && test -d crates/contextra-env
    grep -q 'contextra_ids' crates/contextra-types/src/lib.rs
    grep -q 'contextra_env' crates/contextra-ports/src/lib.rs
    cargo check -p contextra-store --all-targets   (Ergebnis in die Baseline)
Schlägt eines der ersten drei fehl, ist das Fundament-Paket P0 nicht gemergt: Brich ab, schreibe nur docs/handoff/STORE-6.md mit dem Blocker und ändere keinen Code.
```

### Ausführung der START-GATE-Befehle:

1. `test -d crates/contextra-ids && test -d crates/contextra-env`
   - **Ergebnis:** Exit Code 0 (Verzeichnisse existieren).
2. `grep -q 'contextra_ids' crates/contextra-types/src/lib.rs`
   - **Ergebnis:** Exit Code 1 (**FEHLGESCHLAGEN**).
   - Befund: In `crates/contextra-types/src/lib.rs` wird der Bezeichner `contextra_ids` nicht referenziert (Re-Exports erfolgen indirekt über `src/ids.rs`, aber die Startbedingung verlangt zwingend Treffer in `src/lib.rs`).
3. `grep -q 'contextra_env' crates/contextra-ports/src/lib.rs`
   - **Ergebnis:** Exit Code 1 (**FEHLGESCHLAGEN**).
   - Befund: In `crates/contextra-ports/src/lib.rs` wird der Bezeichner `contextra_env` nicht referenziert (Re-Exports erfolgen über `src/clock.rs`, `src/env.rs`, etc., aber nicht direkt in `src/lib.rs`).
4. `cargo check -p contextra-store --all-targets`
   - **Ergebnis:** Exit Code 101 (**FEHLGESCHLAGEN**).
   - Befund: `contextra-store` kompiliert am Ausgangs-HEAD nicht fehlerfrei (u. a. `BlockCacheBackend` Signaturen E0201/E0046 in `block_cache.rs`, `drop(self.file)` E0509 in `builder.rs`, und `contextra_ports::scanner` API-Drifts in `scanner/mod.rs` sowie fehlendes `SstableBuilder::new` in `compaction/tests/basic.rs`).

---

## 2. Handlungsentscheidung gemäß Auftrag

Da die Bedingungen 2 und 3 des START-GATE fehlgeschlagen sind, ist das Fundament-Paket P0 laut Spezifikation formal nicht vollständig konform gemergt.

Gemäß strikter Vorgabe:
> *„Schlägt eines der ersten drei fehl, ist das Fundament-Paket P0 nicht gemergt: Brich ab, schreibe nur docs/handoff/STORE-6.md mit dem Blocker und ändere keinen Code.“*

Es wurde daher kein Code modifiziert und die Bearbeitung nach Erstellung dieses Handoff-Dokuments ordnungsgemäß abgebrochen.

---

## 3. Koordinationsanforderungen

1. **`contextra-types` (`crates/contextra-types/src/lib.rs`)**:
   - Re-Export oder Referenz von `contextra_ids` direkt in `src/lib.rs` ergänzen, sodass `grep -q 'contextra_ids' crates/contextra-types/src/lib.rs` erfolgreich ist.
2. **`contextra-ports` (`crates/contextra-ports/src/lib.rs`)**:
   - Re-Export oder Referenz von `contextra_env` direkt in `src/lib.rs` ergänzen, sodass `grep -q 'contextra_env' crates/contextra-ports/src/lib.rs` erfolgreich ist.
3. **`contextra-store` Baseline-Integrität**:
   - Behebung der bestehenden Kompilierfehler im HEAD, bevor Folgeschritte aufsetzen.
