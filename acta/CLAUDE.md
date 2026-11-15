# CLAUDE.md – Acta

Acta ist eine Wissens- und Entscheidungsplattform mit versionierbarem Unternehmenswissen (Ebene K) und entkoppeltem, nachweisbar löschbarem Personendatenschutz (Ebene P). Die normative Quelle ist **`docs/SPEC.md`**. Bei Widerspruch gilt SPEC 0.4: Zusagen (1.5) vor Invarianten (15.4) vor Verfahren (Kap. 4 bis 14) vor Parametern (Anhang A). Was die Spezifikation nicht erlaubt, ist verboten (Default-deny).

Jeder Crate hat eine eigene `CLAUDE.md` unter `crates/<name>/CLAUDE.md` bzw. `xtask/CLAUDE.md`. Dort stehen Level, Budget, Aufgabe, Abhängigkeiten und Überführungsstand.

## Arbeitsregeln für Claude (verbindlich)

1. **Keine Sub-Agenten außer Haiku.** Der Agent-Tool-Aufruf ist nur mit `model: haiku` erlaubt. Alle Experten in `.claude/agents/` sind auf `model: haiku` gesetzt. Andere Typen (`general-purpose`, `Explore`, `Plan`, `claude` usw.) und jedes andere Modell nicht starten; die Arbeit selbst erledigen. Haiku-Ergebnisse nie ungeprüft übernehmen (Diff lesen, `/gates` selbst ausführen). Aussagen eines Sub-Agenten wie „Tests grün“ zählen erst nach eigenem Lauf.
2. **Lieferung vor Ablauf des Kontingents.** Der Nutzer will alle bearbeiteten Dateien als ZIP. Format: nur bearbeitete oder neue Dateien, Pfade ab Repo-Wurzel (zum direkten Überschreiben); entfernte Dateien als **leere Dateien** mit ihrem Pfad (der Nutzer löscht die Hüllen lokal); nichts anderes im ZIP. Erzeugen mit `bash scripts/make-delivery.sh` (Skill `acta-delivery`, Befehl `/deliver`), ausliefern mit `SendUserFile`.
   - Nach jedem abgeschlossenen Arbeitspaket eine Zwischenlieferung senden, nicht erst am Ende.
   - Sobald Kontext oder Kontingent knapp werden (z. B. ab ca. 80 % Verbrauch, bei Warnhinweisen oder wenn eine lange Aufgabe noch aussteht): sofort liefern, dann weiterarbeiten. Eine nicht gelieferte Arbeit gilt als nicht erledigt.
   - Vor dem Liefern `/gates` laufen lassen und den ehrlichen Stand (grün/rot/nicht geprüft) in die Antwort schreiben.
3. **Ehrlichkeit vor Optik.** Keine Gates, Lints, Budgets oder `deny.toml` lockern, um Rot zu beseitigen. Befunde berichten, Entscheidungen beim Nutzer lassen.

## Umgebung (verifiziert, Stand 2026-11-15)

**Toolchain.** Das Repo verlangt Rust **1.89.0** (`rust-toolchain.toml`). `static.rust-lang.org` ist in der VM blockiert, deshalb wird die Toolchain über conda-forge bereitgestellt:

1. `rust-1.89.0-h53717f1_0.conda` (linux-64) enthält rustc, cargo, clippy, rustfmt, rust-analyzer. Entpacken nach `/root/toolchains/rust-1.89.0`.
2. Die std-Bibliotheken fehlen im obigen Paket. Sie liegen separat in `rust-std-x86_64-unknown-linux-gnu-1.89.0-h2c6d0dc_0.conda` (noarch). Ohne dieses Paket schlägt jeder Build mit `E0463: can't find crate for std` fehl.
3. Einbindung in rustup: `ln -sfn /root/toolchains/rust-1.89.0 /root/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu`. Der Name `1.89.0` allein wird von `rustup toolchain link` abgelehnt.
4. Danach wählt rustup die Toolchain im Repo automatisch. `rustc --version` muss `1.89.0` zeigen.

**Automatik:** Ein SessionStart-Hook (`.claude/settings.json`, `scripts/session-start.sh`) stellt Toolchain (`scripts/bootstrap-toolchain.sh`) und Werkzeuge (`scripts/bootstrap-tools.sh`) sicher und druckt einen Statusbericht. Details, Netztabelle, Befunde und Grenzen: `docs/UMGEBUNG.md`. Skill: `acta-toolchain`.

**Werkzeuge (verifiziert):** just, cargo-deny, cargo-audit, cargo-nextest, cargo-mutants, cargo-llvm-cov, cargo-cyclonedx; cargo-geiger optional (`+stable`). **Nicht beschaffbar:** nightly, Miri, Sanitizer, cargo-fuzz (static.rust-lang.org blockiert). Dazu keine Prüfaussage machen.

**Netz.** `index.crates.io` und `static.crates.io` sind erreichbar, `conda.anaconda.org` ebenfalls. `static.rust-lang.org` und `api.anaconda.org` nicht. Abhängigkeiten werden nur aus crates.io bezogen. Git-Quellen sind laut `deny.toml` verboten.

**Shell.** Das Arbeitsverzeichnis wird zwischen Aufrufen zurückgesetzt. Immer absolute Pfade verwenden oder mit `cd /root/acta &&` beginnen. Lange Läufe (`cargo check --workspace --all-targets` benötigt ca. 25 s auf 2 Kernen, ein Kalt-Build deutlich länger) im Hintergrund starten und das Log auswerten.

## Befehle

```
cargo fetch --locked
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cargo xtask gates          # G1, G-DEF, G-HUB, G-UNSAFE, G-NOTOKIO, G-LINT, G-WORD
cargo nextest run --workspace --locked      # schneller Testlauf
cargo deny check           # G-SUPPLY (Teil): licenses und bans derzeit rot, siehe docs/UMGEBUNG.md
```

Slash-Befehle: `/gates`, `/migrate <crate>`, `/spec <kapitel>`, `/wording`, `/deliver`.

Die `justfile` bündelt dieselben Befehle (`just check`, `just test`, `just lint`, `just fmt`, `just gates`).

**Stand der Prüfungen:** Alle sieben xtask-Gates, `cargo test`, `nextest`, `clippy -D warnings`, `fmt --check`, `cargo deny check advisories sources` grün. **Rot:** `cargo deny check licenses bans` (interne Pfad-Abhängigkeiten ohne Version, `LicenseRef-Proprietary`); Entscheidung und ADR nötig, `deny.toml` nicht eigenmächtig lockern.

## Architektur-Regeln (Kurzfassung, verbindlich: SPEC Kap. 3)

- **Level-Modell (3.2, G1):** Abhängigkeit `A → B` nur wenn `level(A) > level(B)`. Querkanten auf gleichem Level sind verboten. Dev-Abhängigkeiten zählen nicht. Die Level stehen in `rings.toml`.
- **Hub-Regeln HR-1 bis HR-9 (3.4):** kleine Verträge statt Sammel-Crates, ein Fehlertyp je Crate (`thiserror`, `#[non_exhaustive]`, kein `Other(String)`), keine Alias-Re-Exports, `acta-node` ist die einzige Verdrahtungsstelle, `acta-engine` darf nur von `acta-node` genutzt werden (HR-8), `acta-seg` kennt keine Schlüsselverwaltung.
- **Synchroner Kern (3.6):** Level 0 bis 7 sind synchron. `tokio` ist nur ab Level 8 erlaubt (G-NOTOKIO).
- **`unsafe` nur in `acta-sys`** (G-UNSAFE). Jeder Block braucht einen `// SAFETY:`-Kommentar.
- **Dateizugriffe nur in `acta-fs`**, Löschungen nur über den Löschkern, Lesepfade nur über `ErasedView` (G-FS, G-ERA, G-READ, Stand: noch nicht umgesetzt, siehe Abschnitt Offen).
- **Zeit und Zufall nur über Ports** `Clock` und `CryptoRng` (G3, noch nicht umgesetzt).
- **Größenbudgets (3.3, G-HUB):** harte Obergrenzen für Produktionscode ohne Tests. Überschreitung nur mit ADR.
- **Lints (G-LINT):** Workspace-Lints gelten. Auf Produktionspfaden verboten sind `unwrap`, `expect`, `panic`, Indexierung mit `[]` und `let _ =` auf Result.
- **Abhängigkeits-Whitelist (3.8, deny.toml):** nur die dort genannten Crates. Neue Crates brauchen einen ADR-Eintrag. Parser-Abhängigkeiten dürfen nur in `acta-parse` und `acta-parse-worker` vorkommen.

## Sprach- und Berichtsregeln (SPEC 1.6, G-WORD)

Folgende Formulierungen dürfen in Quelltext, Dokumentation, Berichten und Oberflächen nicht vorkommen (Groß-/Kleinschreibung ignoriert): „DSGVO-konform“, „DSGVO-zertifiziert“, „erfüllt Art. 17 automatisch“, „100 % gelöscht“, „garantiert gelöscht“, „garantiert vernichtet“, „unwiederbringlich“, „vergisst Daten“, „revisionssicher“, „fehlerfrei“, „löscht aus allen Backups“, „perfekte Antworten“.

Regel B: Das Wort „gelöscht“ steht in Vorlagen und Oberflächentexten nur zusammen mit der erreichten Erfüllungsklasse („Klasse K erreicht“). Berichte nennen nur `reached` oder `not_reached` mit den Annahmen aus SPEC 20.3. Konformität wird nie als Eigenschaft der Software behauptet.

## Zusagen und Lebenszyklen

- Jede Funktion muss in SPEC Kap. 19 (Lebenszyklen L01 bis L17) stehen. Eine Funktion ohne Eintrag DARF NICHT implementiert werden (SPEC 0.4 Regel 3).
- Zusagen Z1 bis Z19 brauchen mindestens einen grünen Test (G-ZUSAGE, Stand: noch nicht umgesetzt).
- Tests mit Positivkontrolle nach SPEC 16.4.

## Archiv und Überführung

`archive/` enthält Quellcode der Vorgängerimplementierung (837 Dateien, ca. 182 000 Zeilen). **Es ist kein Teil des Cargo-Workspace.** Der Archivcode wird nicht unverändert übernommen. Er wird Crate für Crate nach SPEC in die Architektur überführt:

1. Level-weise von unten nach oben (L0 bis L9). Jedes Crate muss vor dem nächsten grün sein (`cargo check`, `cargo test`, `cargo xtask gates`, Clippy).
2. Archivdateien lesen, Namen und Importe auf Acta umschreiben, Marker `TODO(acta-hr3)` beachten (Typ dem Crate zuordnen, das ihn braucht).
3. Funktionen gegen SPEC Kap. 19 und die Zusagen prüfen. Abweichungen vom Archiv sind zu begründen.
4. **Nach vollständig grüner Überführung eines Crates wird sein Archivordner entfernt.** Die Entfernung erfolgt erst nach der Prüfung, nicht vorher.

Die Überführungsreihenfolge und Arbeitspakete stehen in `README.md` (Abschnitt „Überführung aus dem Archiv“). Details pro Crate stehen in dessen `CLAUDE.md`.

## Stand der Implementierung

- **Implementiert:** `acta-base` (Teil: Ids, Klassen, Fehler, Ports, Festkomma), `acta-testkit` (Teil), `xtask` (sieben Gates).
- **Gerüst mit Fehlertyp:** die übrigen Crates.
- **Offene Gates:** G2, G3, G-CRY, G-FS, G-ERA, G-READ, G-NOSERDE-DISK, G-SUPPLY, G-ZUSAGE.

## Offene Punkte

- Überführung der Archivcodes (alle Level L1 bis L9) nach SPEC.
- Umsetzung der offenen Gates in `xtask`.
- Entscheidung zu `deny.toml` (Wildcard-Pfade, private Lizenz) und Vereinheitlichung `acta-verify` in `rings.toml`.
- Fuzzing und Miri brauchen nightly (in dieser VM nicht verfügbar).

## Experten und Skills

39 Fachexperten als Subagenten (alle `model: haiku`, siehe Arbeitsregel 1) in `.claude/agents/` (je Crate `acta-<crate>-expert`, sieben Querschnittsexperten). Quelle und Pflege: `docs/EXPERTEN-PROMPTS.md`; Auswahl: Skill `acta-experts`.


Projektspezifische Skills liegen unter `.claude/skills/`:
- `acta-toolchain`: Rust 1.89.0 über conda-forge bereitstellen und prüfen.
- `acta-gates`: Architektur-Gates ausführen und Befunde einordnen.
- `acta-archive-migration`: einen Crate aus dem Archiv nach SPEC überführen.
- `acta-spec-check`: Spezifikationsbezug und Wortfilter für Änderungen prüfen.
- `acta-crate-scaffold`: neuen Crate oder neue Abhängigkeit gemäß HR- und Level-Regeln anlegen.
- `acta-experts`: passenden Fachexperten wählen und beauftragen (nur Haiku).
- `acta-delivery`: ZIP-Lieferung der geänderten Dateien erzeugen und ausliefern.
