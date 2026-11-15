---
name: acta-archive-migration
description: "Überführt Archivcode aus archive/<crate>/ sauber nach SPEC in den passenden Acta-Crate unter crates/<crate>/src, prüft ihn und entfernt danach den Archivordner. Verwenden, wenn ein Crate aus dem Archiv implementiert, refaktoriert oder überführt werden soll."
---

# Archiv-Überführung nach SPEC

Das Archiv (`archive/`) ist die Quelle, nicht das Ziel. Es ist kein Teil des Workspace. Der Code wird nicht unverändert kopiert. Er wird gegen `docs/SPEC.md` gelesen, umgeschrieben und geprüft. Nach erfolgreicher Überführung wird der Archivordner des Crates entfernt.

## Experten
Arbeite mit dem Subagenten des Ziel-Crates (`.claude/agents/<crate>-expert.md`, Skill `acta-experts`) und starte per `/migrate <crate>`. Das Expertenprofil steht in `docs/EXPERTEN-PROMPTS.md`.

## Vorbedingungen

1. Die Wurzel-`CLAUDE.md`, die `CLAUDE.md` des Ziel-Crates und `archive/INDEX.md` lesen.
2. Prüfen, dass alle Crates niedrigeren Levels bereits grün sind (`cargo check`, `cargo test`, `cargo xtask gates`). Überführung läuft Level-weise von L0 nach L9.
3. Den Ziel-Crate in `rings.toml` nachschlagen: Level, Budget.
4. `archive/<crate>/MANIFEST.md` lesen. Dort stehen Dateiliste, LOC und Altquellen außerhalb der Whitelist.

## Schritte

### Schritt 1: Spezifikation sammeln
- Kapitel aus der Tabelle in SPEC 3.3 und die Verweise im Kopf der `lib.rs` des Ziel-Crates lesen.
- Alle Lebenszyklen aus SPEC Kap. 19 heraussuchen, die diesen Crate betreffen. Funktionen ohne Eintrag werden nicht übernommen (SPEC 0.4 Regel 3).
- Relevante Zusagen aus 1.5 und Tests aus 16.3 notieren.

### Schritt 2: Archivdateien prüfen
Für jede Datei klären:
- Gehört sie in den Ziel-Crate? Wenn nicht, zu dem Crate verschieben, der sie braucht (HR-2 und HR-3). Marker `TODO(acta-hr3)` zeigt solche Stellen.
- Ist die Abhängigkeit in der Whitelist (SPEC 3.8)? Wenn nicht: nicht übernehmen, ADR-Eintrag vorschlagen.
- Verwendet sie `SystemTime::now`, `thread_rng` oder direkte Dateizugriffe? Dann auf Ports umstellen (`Clock`, `CryptoRng`, `Vfs`). G3 und G-FS verlangen das.
- Enthält sie `serde` oder `bincode` für Plattenformate? Verboten (G-NOSERDE-DISK, SPEC 3.8).

### Schritt 3: Umschreiben
- Namen und Importe auf Acta-Crates anpassen.
- Verweise auf frühere Spezifikationsstände entfernen (im Archiv bereits entfernt, auf Reste prüfen).
- Fehlertyp: ein Typ je Crate, `thiserror`, `#[non_exhaustive]`, kein `Other(String)`.
- Keine `unwrap`, `expect`, `panic`, Indexierung auf Produktionspfaden.
- `unsafe` nur in `acta-sys` und mit `// SAFETY:`.
- Wortfilter SPEC 1.6 beachten.

### Schritt 4: Tests und Positivkontrollen
- Unit-Tests aus dem Archiv übernehmen, soweit sie zur neuen Schnittstelle passen.
- Für jede Zusage aus SPEC 16.3 mindestens einen Test anlegen.
- Positivkontrolle nach SPEC 16.4: Ein Test muss nachweisen, dass er einen absichtlich eingebauten Fehler erkennt.

### Schritt 5: Prüfung
```
cargo check -p <crate> --all-targets --locked
cargo test -p <crate> --locked
cargo clippy -p <crate> --all-targets --locked -- -D warnings
cargo xtask gates
cargo fmt --all -- --check
```
Alle Befehle müssen mit Exit 0 enden.

### Schritt 6: Archiv entfernen
Erst wenn Schritt 5 grün ist:
- `archive/<crate>/` entfernen.
- Zeile in `archive/INDEX.md` anpassen (als überführt markieren).
- `crates/<crate>/CLAUDE.md` auf Status „implementiert“ setzen.
- Wurzel-`CLAUDE.md`, Abschnitt „Stand der Implementierung“, aktualisieren.

Wenn das Entfernen per Auslieferung als Datei-Hülle erfolgen soll, die entfernten Dateien als leere Dateien mit ihrem Pfad ausliefern, damit sie lokal gelöscht werden können.

## Nicht tun

- Archivcode blind kopieren.
- Ein Gate abschwächen, um die Überführung durchzubekommen.
- Budgets in `rings.toml` erhöhen, ohne ADR.
- Das Archiv löschen, bevor der Crate grün ist.
