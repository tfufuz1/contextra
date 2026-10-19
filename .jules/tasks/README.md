# Task-Karten in Contextra

Eine Task-Karte beschreibt ein isoliertes, maschinell prüfbares Arbeitspaket für Jules. Sie verhindert vage Aufträge und setzt klare Scope- und Budgetgrenzen.

## Aus Issue zu Karte (ADR-N30)

Künftig wird die Eingabe für Jules über ein GitHub-Issue per Issue-Form bereitgestellt. Aus den strukturierten Feldern des Issues leitet Jules die Task-Karte als `.toml`-Datei ab.

### ID-Schema
Jede aus einem GitHub-Issue abgeleitete Task-Karte verwendet das ID-Schema:
`T-2026-<Issue-Nummer, 4-stellig>` (z. B. `T-2026-9004` für Issue #9004).

### Vertrag: Issue-Form-Feld → Karten-Feld

Die folgende Tabelle beschreibt den verbindlichen Vertrag zwischen den Feldern der Issue-Form und den Feldern der Task-Karte. Jede Feldprüfung ist durch den xtask-Harness abgesichert:

| Issue-Form-Feld | Task-Karten-Feld (TOML) | Typ & Format | Validierung / xtask-Code-Beleg |
| :--- | :--- | :--- | :--- |
| **Crate** | `crate` | String | Pflichtfeld (`xtask/src/harness/task_card.rs:18-20,162`). Muss in `capabilities.toml` unter `[crates.<name>]` existieren (`xtask/src/harness/task_card.rs:172`). |
| **Prompter Tier** | `tier` | Integer (`u32`) | Optional (`xtask/src/harness/task_card.rs:21`). Wenn in `.jules/prompter-tiers.toml` definiert, muss der Wert übereinstimmen (`xtask/src/harness/task_card.rs:184`). |
| **Risiko-Klasse** | `risk` | String | Pflichtfeld (`xtask/src/harness/task_card.rs:22`). Zulässige Werte: `sec`, `crash`, `simd`, `utf8`, `fusion`, `concurrency`, `calib`, `none`. Bei `risk != "none"` ist `evidence_required` Pflicht (`xtask/src/harness/task_card.rs:233`). |
| **Ziel** | `goal` | String | Pflichtfeld (`xtask/src/harness/task_card.rs:23`). Darf nicht leer sein (`xtask/src/harness/task_card.rs:158`). |
| **Nicht-Ziele** | `non_goals` | Array von Strings | Pflichtfeld (`xtask/src/harness/task_card.rs:24,160`). Beschreibt explizite Scope-Ausschlüsse. |
| **Erlaubter Scope** | `scope` | Array von Globs | Pflichtfeld (`xtask/src/harness/task_card.rs:25`). Darf nicht leer sein (`xtask/src/harness/task_card.rs:202`). Keine Vollglobs (`**`, `crates/**`, `crates/*`, `xtask/src/harness/task_card.rs:211`). Keine Überschneidung mit `forbidden` (`xtask/src/harness/task_card.rs:221`). Wird von `scope-guard` ausgewertet (`xtask/src/harness/scope_guard.rs:31`). |
| **Verbotene Pfade** | `forbidden` | Array von Globs | Pflichtfeld (`xtask/src/harness/task_card.rs:26`). Muss alle geschützten Pfade (`.github/**`, `xtask/**`, `capabilities.toml`, `AGENTS.md`, `rust-toolchain.toml`, `deny.toml`, `.jules/setup/**`) enthalten (`xtask/src/harness/task_card.rs:52`), es sei denn `protected_change` ist gesetzt (`xtask/src/harness/task_card.rs:227`). Wird von `scope-guard` ausgewertet (`xtask/src/harness/scope_guard.rs:33`). |
| **Invarianten** | `invariants` | Array von Strings | Pflichtfeld (`xtask/src/harness/task_card.rs:27`). Warnung bei leerer Liste (`xtask/src/harness/task_card.rs:167`). |
| **Akzeptanzkriterien** | `acceptance` | Array von Befehlen | Pflichtfeld (`xtask/src/harness/task_card.rs:28`). Darf nicht leer sein (`xtask/src/harness/task_card.rs:224`). Jeder Befehl muss mit `cargo ` oder `just ` beginnen (`xtask/src/harness/task_card.rs:229`). |
| **Erforderliche Belege** | `evidence_required` | Array von Strings | Pflichtfeld bei `risk != "none"` (`xtask/src/harness/task_card.rs:29,233`). Bei `risk = "none"` optional/leer zulässig. |
| **Datei-Budget** | `budget.files` | Integer (`u32`) | Pflichtfeld (`xtask/src/harness/task_card.rs:11`). Maximal 25 Dateien (`xtask/src/harness/task_card.rs:242`). Standard-Fallback im Diff-Budget-Gate: 15 (`xtask/src/harness/diff_budget.rs:25,107`). |
| **Zeilen-Budget** | `budget.lines` | Integer (`u32`) | Pflichtfeld (`xtask/src/harness/task_card.rs:12`). Maximal 1000 Zeilen (`xtask/src/harness/task_card.rs:253`). Standard-Fallback im Diff-Budget-Gate: 600 (`xtask/src/harness/diff_budget.rs:26,108`). |
| **Iterations-Budget** | `budget.iterations` | Integer (`u32`) | Pflichtfeld (`xtask/src/harness/task_card.rs:13`). Maximal 12 Iterationen (`xtask/src/harness/task_card.rs:264`). |
| **Issue-Nummer** (Optional) | `issue` | Integer / String | Optionales Zusatzfeld. Da `TaskCard` kein `#[serde(deny_unknown_fields)]` aufweist (`xtask/src/harness/task_card.rs:15`), wird das Feld vom Serde-Deserialisierer akzeptiert. |

### Automatische Kartenerzeugung aus dem Issue-Body

1. **Erzeugung im PR:** Wenn Jules einen Auftrag aus einem GitHub-Issue bearbeitet (z. B. Issue #9004), erzeugt Jules die entsprechende Task-Karten-Datei selbst als `.jules/tasks/T-2026-9004.toml` innerhalb der PR-Commits.
2. **Scope-Guard-Konformität:** Der in der Karte definierte `scope` muss die Karten-Datei `.jules/tasks/T-2026-9004.toml` (sowie alle weiteren geänderten Dateien) erlauben (`xtask/src/harness/scope_guard.rs:175`).
3. **PR-Referenzierung:** Der PR-Body bzw. die Git-Commit-Trailers müssen die Karte über die Zeile `Task-Card: T-2026-9004` referenzieren, damit die Gates `scope-guard` (`xtask/src/harness/scope_guard.rs:118`), `diff-budget` (`xtask/src/harness/diff_budget.rs:72`) und `task-card lint` (`xtask/src/harness/task_card.rs:285`) die Karte auffinden und validieren können.

### Aufräumregel für erledigte Karten

- **Persistierung nach Merge:** Nach dem erfolgreichen Merge eines PRs verbleibt die Task-Karte in `.jules/tasks/` als unmanipulierbarer Nachweis (Audit Trail) und zur Historienverfolgung.
- **Bereinigung von Entwürfen:** Unvollständige oder verworfene Entwurfs-Karten ohne gemergten PR sind vor dem finalen Commit aufzuräumen bzw. zu entfernen.

## Beispiele für Task-Karten

### Beispiel 1: WAL-fsync Fehlerbehandlung (`crates/contextra-store`)
```toml
id        = "T-2026-0101"
title     = "WAL: fsync-Fehler propagieren statt verschlucken"
crate     = "contextra-store"
tier      = 1
risk      = "crash"
goal      = "Ersetze ignoriertes fsync in wal/flusher.rs durch fehlerfortpflanzendes Result."
non_goals = ["Keine Änderungen am CompactionEngine", "Keine neuen Dependencies"]
scope     = ["crates/contextra-store/src/wal/flusher.rs", "crates/contextra-store/tests/wal_*.rs"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["INV-WAL-POISON-EXPLICIT", "Zero-Panic"]
acceptance = [
  "cargo test -p contextra-store --test wal_lifecycle_crash_point_matrix"
]
evidence_required = ["Neuer Test für I/O-Fehler beim Flushen"]

[budget]
files = 6
lines = 250
iterations = 6
```

### Beispiel 2: SIMD-Bounds-Prüfung (`crates/contextra-simd`)
```toml
id        = "T-2026-0102"
title     = "SIMD: Slices-Bounds-Check vor Vektorisierung sicherstellen"
crate     = "contextra-simd"
tier      = 1
risk      = "simd"
goal      = "Expliziten Slices-Längencheck vor SIMD-Dot-Product einfügen."
non_goals = ["Keine ungesicherten Memory-Layouts"]
scope     = ["crates/contextra-simd/src/lib.rs"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Zero-Panic", "Unsafe Islands Only"]
acceptance = [
  "cargo test -p contextra-simd"
]
evidence_required = ["Fuzzing-Test oder Out-of-bounds Testfall"]

[budget]
files = 3
lines = 100
iterations = 4
```

### Beispiel 3: Dokumentations-Fix (`crates/contextra-types`)
```toml
id        = "T-2026-0103"
title     = "Doku: Sync von Domain-Type Invarianten"
crate     = "contextra-types"
tier      = 1
risk      = "none"
goal      = "Aktualisiere Doc-Comments in domain/mod.rs bezüglich Fehlercodes."
non_goals = ["Keine API-Code-Änderungen"]
scope     = ["crates/contextra-types/src/types/domain/mod.rs"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Single Error Root"]
acceptance = [
  "cargo test -p contextra-types"
]
evidence_required = ["Doku-Check bestanden"]

[budget]
files = 2
lines = 50
iterations = 3
```

### Beispiel 4: Aus Issue #9004 abgeleitete Task-Karte (`crates/contextra-core`)
```toml
id        = "T-2026-9004"
title     = "docs(jules): Task-Karten v2, Issue-kompatibel"
crate     = "contextra-core"
tier      = 1
risk      = "none"
issue     = 9004
goal      = "Aktualisiere .jules/tasks/README.md und _TEMPLATE.toml für Issue-Form-Kompatibilität (ADR-N30) und erstelle Task-Karte T-2026-9004."
non_goals = ["Keine Änderungen an xtask/**", "Keine Änderungen an .github/**"]
scope     = [".jules/tasks/README.md", ".jules/tasks/_TEMPLATE.toml", ".jules/tasks/T-2026-9004.toml"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Zero-Panic"]
acceptance = [
  "cargo test -p contextra-core"
]
evidence_required = []

[budget]
files = 3
lines = 300
iterations = 5
```
