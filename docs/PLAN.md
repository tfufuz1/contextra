# Contextra – Umsetzungsplan 4.1

| Feld | Wert |
|---|---|
| Stand | 31.10.2026 |
| Normativ | `docs/SPEC.md` Fassung 4.1 |
| Ersetzt | Plan 4.0 (Phasen R0 bis R6). Archiv: `docs/spec/archiv/PLAN-4.0.md` |
| Ausgangscommit | `3a0b11699` |
| Zeitangaben | Orientierung für einen Entwickler. Verbindlich sind Reihenfolge und Abnahmekriterien |

Dieser Plan ersetzt den Reduktionsplan 4.0. Er geht davon aus, dass alle 35 Crate-Verzeichnisse im Repo liegen. Die Reduktion ist eine Entscheidung (D-K1), keine Voraussetzung mehr.

---

## K0: Build herstellen (Blocker, Ziel: 1 bis 2 Wochen)

Ohne K0 ist kein anderer Schritt prüfbar. Abnahme: Build-Vertrag B1 bis B7 aus SPEC §4 ist grün.

| Schritt | Aufgabe | Abnahme |
|---|---|---|
| K0.1 | `xtask` entscheiden. Hintergrund: Der Gate-Runner wurde mit `068261abd` absichtlich entfernt (`CLAUDE.md`), der Workspace-Eintrag blieb stehen. Option a: Gate-Runner aus `4c1d3a675` wiederherstellen (dort 1.311 Funktionen, Stand 29.10.) und gegen die Gates in `governance/gates.toml` prüfen. Option b: Member aus `Cargo.toml` entfernen und `governance/gates.toml` bereinigen. Empfehlung: Option b, sofern die Gates nicht als Abnahme genutzt werden sollen; sonst Option a | `cargo metadata --locked` läuft durch |
| K0.2 | Feature `onnx`: in `contextra-engine/Cargo.toml` `onnx = ["dep:contextra-infer-onnx"]` setzen und `contextra-infer-onnx = { workspace = true, optional = true }` ergänzen | `cargo build -p contextra-engine --features onnx` |
| K0.3 | Feature `adaptive-candidate-pool-sizing`: analog mit `dep:contextra-adapt` | `cargo build -p contextra-engine --features adaptive-candidate-pool-sizing` |
| K0.4 | Gesamtbuild: `--workspace --locked`, `--all-features`, `--no-default-features` | Drei Läufe grün oder Fehler als Issue erfasst |
| K0.5 | Tests und Lints: B5 bis B7 | 4.729 Tests laufen, Fehlschläge als Liste mit Ursache |
| K0.6 | CI-Workflows (`rust-ci.yml`, `merge-gate.yml`) auf dem Stand ausführen | Grüne Läufe, Link im Changelog |
| K0.7 | Vector-I/O (Befund A3): Persistenzpfad in `contextra-vector` hinter einen Port legen, oder Crate-Ausnahme mit ADR | `check_ring0_async_purity` prüft den Fall |

Hinweis zu K0.1: Die Version in `4c1d3a675` passt womöglich nicht zu den Crates, die am 30. bis 31.10. zurückgekommen sind. Nicht blind übernehmen, sondern gegen den Build prüfen.

## K1: Scope-Entscheidung (Ziel: vor Ende K0)

Entscheidung D-K1 durch ADR 0009. Abnahme: Status ist `Accepted`, SPEC §3.2 ist ohne Option aktualisiert.

| Schritt | Aufgabe |
|---|---|
| K1.1 | Liste der 35 Crates mit Nutzen-Notiz versehen (Vorlage: `ARCHITECTURE.md` §3) |
| K1.2 | Für jedes Crate Disposition wählen: Kern, Produkt, Lab, Archiv-Tag |
| K1.3 | ADR 0009 mit Optionen und Begründung schreiben, Status `Accepted` setzen |
| K1.4 | `capabilities.toml` an die Entscheidung anpassen |

Empfehlung zur Entscheidung (kein Beschluss): Option B mit Reifegrad-Gating ist der kleinste Schritt. Sie löscht nichts, und jede Fähigkeit ist über `capabilities.toml` sichtbar. Option A ist sinnvoll, wenn in den nächsten zwei Monaten nur eine Produktlinie entwickelt wird.

## K2: Dokumentation synchronisieren (Ziel: parallel zu K0)

| Schritt | Aufgabe |
|---|---|
| K2.1 | README: Statusabschnitt durch Verweis auf SPEC §2 ersetzen, Crate-Zahl 35 |
| K2.2 | `CLAUDE.md` und `AGENTS.md` überarbeiten. Widersprüche am Commit: `CLAUDE.md` verbietet das Anlegen und Referenzieren von 20 Crates, die im Repo liegen (Abschnitt „Entfernte Crates“). `AGENTS.md` beschreibt `cargo xtask` als gültig und zugleich als entfernt. Beide Dateien sind in dieser Version nicht überarbeitet |
| K2.3 | Generator für die Crate-Tabelle in `ARCHITECTURE.md` schreiben (Skript liest `Cargo.toml` und `capabilities.toml`), damit die Tabelle nicht von Hand gepflegt wird |
| K2.4 | `CHANGELOG.md` mit Eintrag 4.1 ergänzen |
| K2.5 | `contextra-durable-fs` in `capabilities.toml` eintragen (Ring, Reifegrad, Unsafe-Insel). Prüfung in CI: jedes Crate-Verzeichnis hat einen Eintrag, und die Crate-Zahl in README und ARCHITECTURE stimmt mit dem Workspace überein |

## K3: Löschkette klären (nur nach K1 und ADR)

Wird die Löschkette für Option B als Teil des Produkts geführt, gilt:

| Schritt | Aufgabe |
|---|---|
| K3.1 | Layer-Liste aus Fassung 3.1 §9 gegen Code abgleichen (Referenzabgleich 29.10. als Ausgang, nicht als Wahrheit) |
| K3.2 | Ungemessene Schichten (`UNMEASURED_ERASURE_LAYERS` in `contextra-engine`) mit Messung oder Ausschluss versehen |
| K3.3 | Testfälle: nach Löschung keine Reste in Embeddings, Graphkanten, Caches, WAL und SSTables |
| K3.4 | Erst bei bestandener Messung Reifegrad auf `beta` setzen. `stable` nur mit externem Review |

## K4: Produktlinien (nach K1)

Die drei Produkte aus dem Entscheidungsdokument v2 (Shield, Retrieve, Trace) bleiben Optionen. Sie werden erst nach K0 bis K2 begonnen. Produktbeschreibungen und Go/No-Go-Kriterien stehen in `contextra_produktaufteilung_v2.md` §3.2 und §8.

| Produkt | Vorbedingung |
|---|---|
| Retrieve (P2-Lite: `ingest`, `text`, `rank`) | K0 grün, Chunker aus `engine` gelöst (E1) |
| Trace (P3: `checkpoint`) | K0 grün, `checkpoint` ohne `core` (bereits erfüllt) |
| Shield (P1) | K0 grün, Proxy-Schicht als Neubau, Crypto-Verdrahtung in der Composition Root |

## Risiken und Abbruchkriterien

| Risiko | Gegenmaßnahme |
|---|---|
| `xtask`-Wiederherstellung bricht Gates | K0.1 mit Option b als Fallback, Abnahme nach jedem Schritt |
| Feature-Fix zieht weitere undeklarierte Abhängigkeiten nach sich | Suche nach `contextra_` in allen `src/`, nicht nur in den zwei bekannten Stellen |
| Build bleibt rot über mehr als 3 Wochen | Eskalation: Scope auf Option A reduzieren, Rest per Archiv-Tag |
| Nicht erneut geprüfte Marktdaten | Vor K4 neu recherchieren |
