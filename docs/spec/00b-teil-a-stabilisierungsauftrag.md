---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "00b"
---
## Teil A — Stabilisierungsauftrag: Ground Truth, Reifegrade, Gates

> **Rekonstruktionsvermerk (Fassung 2.1):** Teil A ist im Kopf dieses Dokuments und in Teil A2 als ranghöchster
> Teil (§A.1–§A.5) referenziert, stand in Fassung 2 aber nicht im Dokument. Dieser Teil ist aus den Aussagen
> rekonstruiert, die das Dokument selbst über Teil A macht (Kopf, §A2, §16, §17, §18, §20). Wo diese Aussagen
> nichts festlegen, ist die Festlegung mit **[F2.1]** gekennzeichnet und vom Product Owner zu bestätigen. Liegt
> die ursprüngliche Fassung vor, ersetzt sie diesen Teil vollständig.

### A.1 Rang und Ground-Truth-Regel

Teil A ist ranghöher als alle übrigen Teile (Ausnahme-Reihenfolge laut Kopf: Ground-Truth-Herstellung vor
Struktur-Migration vor allen übrigen Abschnitten). **Ground Truth** ist der Zustand, in dem für einen Crate
gilt: ein frischer, an genau einen Commit gebundener CI-Lauf liegt vor, und alle Reifegrad-Marker des Crates
wurden aus diesem Lauf erzeugt. **Kein Feature-Ausbau, bevor Ground Truth hergestellt und das Fundament
(Ring 0–1) nachweisbar stabil ist.**

### A.2 Reifegrad-Marker (verschärfte Bedeutung)

- 🟢 gilt nur, wenn ein frischer, commit-gebundener CI-Lauf am aktuellen HEAD den Sachverhalt bestätigt.
  Andernfalls ist die Markierung eine Behauptung aus einer Vorfassung und wird wie 🔍 behandelt.
- 🔍 (Nachverifikation ausstehend) ist der Zustand jedes aus einer Vorfassung übernommenen Markers, bis Gate 0
  für den betroffenen Crate durchlaufen ist. Contributor MÜSSEN 🟢/🔴 bis dahin faktisch als 🔍 behandeln.
- Ein 🟢 wird widerrufen, sobald Teil A2 einen belegten Gegenbefund (D#, §A2.1) nennt oder ein CI-Lauf ihn
  widerlegt. **[F2.1]** Marker werden maschinell aus `capabilities.toml` und den CI-Ergebnissen erzeugt und
  nicht von Hand gepflegt (P12, §20).

### A.3 Phasen und Gates **[F2.1]**

Jeder Abschnitt außerhalb von Teil A trägt implizit `[Phase 1]`, sofern er keine andere Kennzeichnung führt.
Die Phasen entsprechen den Stufen der Roadmap (§17, §18): Phase *n* ↔ Stufe *n − 1*.

| Phase | Inhalt | Eintrittsbedingung |
|---|---|---|
| 1 | Fundament (Ring 0–1): Korrektheit und Betriebssicherheit (Stufe 0) | Gate 0 für den betroffenen Crate |
| 2 | Hot-Path-Performance (Stufe 1) | Gate 1 |
| 3 | Speicher und Struktur (Stufe 2) | Gate 2 |
| 4 | Governance und Produktentscheidungen (Stufe 3) | Gate 3 |
| 5 | Fernziele (§18, Stufe 4) | Gate 4 |

- **Gate 0 (je Crate):** frischer, commit-gebundener Lauf von `cargo fmt --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings` und `cargo test -p <crate> --locked`, grün am
  aktuellen HEAD; Marker des Crates neu erzeugt (§A.2).
- **Gate 1 (Fundament stabil):** Gate 0 für alle Ring-0/1-Crates; Kernkriterien K-* aus §16.1, soweit Stufe 0;
  Loom-Tests aus §15.3 grün; Layering-Test (§4.3) scharf.
- **Gate 2 bis 4:** alle in der jeweils vorangehenden Phase genannten Abnahmekriterien (§16) grün, am selben
  Commit wie der Gate-Nachweis.

### A.4 Diagnose-Artefakte und Commit-Bindung

Testergebnisse, Lint-Reports, Audit-Dokumente und Marker-Tabellen tragen im Kopf `commit: <SHA>`,
`generated_by: <Befehl>`, `generated_at: <UTC>` und `toolchain: <rustc-Version>`. Ein Artefakt mit
`commit ≠ HEAD` ist **veraltet** und darf nicht als Begründung für Arbeit oder Nicht-Arbeit dienen. Artefakte
werden mechanisch (xtask/CI, 🔴 zu bauen) erzeugt, nie von Hand editiert.

### A.5 Änderungsdisziplin

Arbeit an einem Abschnitt der Phase *n* beginnt erst nach Gate *n − 1*. Ausgenommen sind Korrekturen mit
Stufe-0-Charakter (Panic, Datenverlust, Sicherheitslücke) und reine Dokumentationsänderungen. Beschlossene
ADRs (§20.3) und Teil-A2-Migrationsschritte (§20.2) gelten als Phase-1-Arbeit, soweit sie Ring 0–1 betreffen.

---

<a id="a2-zielarchitektur-v2"></a>
