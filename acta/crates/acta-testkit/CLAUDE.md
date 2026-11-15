# acta-testkit

acta-testkit: `ManualClock`, `SeededRng`, Fehlerinjektion, Konformitaetstests (Spec 16.1, 16.2).

> Teil des Acta-Workspace. Verbindliche Vorgaben: Wurzel-`CLAUDE.md` und `docs/SPEC.md`.

## Einordnung
- **Level:** 2 (`rings.toml`)
- **Größenbudget:** – LOC Produktionscode (G-HUB); Überschreitung nur mit ADR
- **Aufgabe laut SPEC 3.3:** `ManualClock`, `SeededRng`, Fehlerinjektion, Konformitätstests, Testdaten
- **Erlaubte interne Abhängigkeiten (SPEC 3.3):** base, crypto, fs
- **Ist-Abhängigkeiten (Cargo.toml):** acta-base, acta-crypto, acta-fs
- **Externe Abhängigkeiten:** keine
- **Dev-Abhängigkeiten (zählen nicht für G1):** keine
- **Referenz im Quelltext:** —

## Status
- **Stand:** implementiert (Teil)
- **Produktionscode (ohne Tests/Leerzeilen/Kommentare):** 155 LOC
- **Umgesetzt:** `ManualClock`, `SeededRng`, `FailingRng` (Kern); `conformance/failure` und `conformance/macros` (1:1 aus dem Archiv, nur Dokumentation auf SPEC 16.1 umgestellt); `conformance/crypto_rng` (neu, auf den Vertrag `CryptoRng::try_fill` ausgerichtet, Z10). Die Archiv-Vorlage `rng.rs` nutzt die alte API `next_u64`/`fill_bytes` und wurde nicht übernommen.
- **Bewusst nicht umgesetzt:** Clock-Suite aus dem Archiv. Der Port garantiert nach A-CLOCK nur `now_ms`, keine Monotonie. Die Archiv-Suite würde eine nicht zugesicherte Eigenschaft prüfen.
- **Offen:** Suiten für `Vfs`, Textindex, Vektorindex und Speicher (im Archiv fehlen die Quellen `conformance/{vfs,text,vector,storage}`). Fehlerinjektion `FaultVfs` liegt in `acta-fs` (`acta_fs::fault`).
- **Tests:** Einheitstests in den Modulen und Integrationstest `tests/conformance_export.rs` (Prüfung über die exportierte Crate-Wurzel).
- **Archiv-Quelle:** `archive/acta-testkit/` (siehe `archive/acta-testkit/MANIFEST.md`). Der Archivordner bleibt bestehen, bis das Crate vollständig überführt ist (CLAUDE.md, Abschnitt Archiv und Überführung).

## Regeln für diesen Crate
- Abhängigkeiten nur nach unten: `level(A) > level(B)`; keine Querkanten (G1).
- Keine Alias-Re-Exports, kein `include!`, kein `#[deprecated]` (HR-6).
- Ein Fehlertyp je Crate, `thiserror`, `#[non_exhaustive]`, kein `Other(String)` (HR-4).
- Workspace-Lints gelten (`[lints] workspace = true`): kein `unwrap`, `expect`, `panic`, Indexierung mit `[]` auf Produktionspfaden.
- `unsafe` ist hier verboten (G-UNSAFE, nur `acta-sys` darf es).
- Keine Zeit- oder Zufallsquellen außerhalb der injizierten Ports `Clock` und `CryptoRng` (G3).
- Verbotene Formulierungen aus SPEC 1.6 (Liste A) nicht verwenden (G-WORD).
- Vor Abschluss: `cargo xtask gates`, `cargo test -p acta-testkit`, `cargo clippy -p acta-testkit --all-targets -- -D warnings`, `cargo fmt --all -- --check`.

## Vorgehen bei Überführung aus dem Archiv
1. Archivdateien lesen, nicht blind kopieren. Namen, Importe und Verweise auf frühere Spezifikationsstände sind auf Acta umzuschreiben (siehe `archive/INDEX.md`).
2. Jede Funktion gegen den Lebenszyklus in SPEC Kap. 19 und die Zusagen in 1.5 und 16.3 prüfen. Funktionen ohne Eintrag dort werden nicht implementiert (SPEC 0.4 Regel 3).
3. Tests mit Positivkontrolle mitführen (SPEC 16.4).
4. Nach vollständiger, grün geprüfter Überführung wird der Archiv-Ordner dieses Crates entfernt.
