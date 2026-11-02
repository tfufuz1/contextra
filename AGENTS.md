# Contextra – Hinweise für Agenten

Diese Datei ist die Agentenanweisung des Repositorys. Sie ist **nicht normativ**. Normativ ist ausschließlich [`docs/SPEC.md`](docs/SPEC.md) (Version 4.2.0). Bei jedem Widerspruch gilt die SPEC; jede Abweichung ist ein Defekt.

## 1. Was Contextra ist

Eingebettete Rust-Bibliothek (Single-Node, kein Server) für die lokale Speicherung von Wissen und Gedächtnis von Agenten- und RAG-Systemen: Dokumente, BM25-Volltextsuche, HNSW-Vektorsuche, RRF-Hybridsuche, MVCC-Zeitreisen und ein gemessener, unabhängig prüfbarer Löschprozess (SPEC 1.1).

## 2. Ist-Zustand

- Der Workspace **baut derzeit nicht** (`cargo check --workspace --all-targets --locked` bricht ab, SPEC F-32).
- Der Löschkern `crates/contextra-erasure` ist **leer** und kein Workspace-Mitglied (SPEC F-33). Die Kernmission ist im Code noch nicht vorhanden.
- Die Arbeit folgt den Arbeitspaketen in SPEC 15.4 und den Gates in SPEC 15.3. Reihenfolge: A → B → G0 → C → D → F → E → G → H; G-LEGAL läuft parallel.

## 3. Harte Regeln

Vollständig: SPEC 6.3 und Kapitel 2. Die wichtigsten Regeln:

- Kein Netzwerkzugriff, keine Telemetrie, kein Schlüssel-Escrow (HR-1 bis HR-3, D-43, D-54).
- Verschlüsselung und WAL-Integrität sind nicht abschaltbar. Kein Feature, kein Laufzeitparameter und kein Umgebungsschalter schwächt sie (P5, D-05, HR-13).
- Kein `unwrap`, `expect`, `panic!`, `unreachable!`, `let _ =` auf `Result` und kein `unwrap_or_default` auf I/O im Produktionscode (P6).
- `unsafe` nur in `contextra-sys` und `contextra-simd` (D-29, AR-8). Keine Speicherabbildung (`mmap`, D-94).
- I/O ausschließlich über den Port `Vfs` (D-04). Kein `std::fs` und kein `tokio::fs` außerhalb der Implementierung in `contextra-durable-fs`.
- Zeit, Zufall und IDs nur über die Ports `Clock`, `Rng` und `IdGen` (P7).
- `fsync`-Fehler werden nie auf demselben Handle wiederholt (D-55).
- Keine Löschaussage außer der zulässigen Außenaussage in SPEC 6.4 (HR-5, T-LINT-1). Die verbotenen Begriffe stehen in SPEC 6.4.
- Im Quelltext und in Dokumenten keine Agenten-, Sitzungs- oder Zeitstempelmarken und keine absoluten Entwicklerpfade (D-91).
- Neue `unsafe`-Blöcke, Abhängigkeiten oder Features nur mit Nennung des Tests, der sie belegt (HR-12).
- Jeder Test trägt `#[ctx_test(id = "T-…")]`; jede Benchmark `#[ctx_bench(id = "BM-…")]` (T-TRACE-1, SPEC 15.1).

## 4. Ringe und Abhängigkeiten

Abhängigkeiten zeigen nur auf gleiche oder niedrigere Ringe (SPEC 5.2). Erlaubte Kanten stehen in der Matrix SPEC 5.3. Vor jeder neuen Abhängigkeit die Matrix prüfen (AR-1, AR-14).

## 5. Arbeitsweise

- Ein PR bearbeitet ein Arbeitspaket aus SPEC 15.4. Vor der Implementierung steht ein fehlschlagender Test mit SPEC-ID (D-44, P10).
- `Cargo.lock` wird im selben Commit wie jede Manifest-Änderung erneuert. Prüfungen laufen mit `--locked` (D-93).
- Befehle und Gates: `docs/COMPILATION.md`. Regeln für Beiträge: `docs/CONTRIBUTING.md`.
- Ändert sich ein Fakt, wird zuerst die SPEC geändert (Anhang H), danach die abgeleiteten Dokumente.
- Ist eine Anforderung unklar, wird nicht geraten: Die betroffene SPEC-ID nennen und nachfragen (P2, fail-closed).

## 6. Wo steht was

| Thema | Datei |
|---|---|
| Normative Spezifikation | `docs/SPEC.md` |
| Architektur, Ringe, Datenflüsse | `docs/ARCHITECTURE.md` |
| Typen und ihre kanonischen Orte | `docs/TYPE_REGISTRY.md` |
| Verbindliche Vetos | `docs/VETOES.md` |
| Grundsätze und Governance | `docs/CONSTITUTION.md` |
| Build, Prüfung, Gates | `docs/COMPILATION.md` |
| Beitrag und Prozess | `docs/CONTRIBUTING.md` |
| Sicherheit und Meldeweg | `docs/SECURITY.md` |
| Rangfolge der Quellen | `docs/SOURCE_OF_TRUTH.md` |
| Crate-Hinweise | `crates/*/AGENTS.md` |



---

<SYSTEMPROMPT: GOOGLE-JULES>

Als Jules initialisierst du die Aufgaben mit Planungen zu einem konkreten Plan und sobald ich diesen Bestätige führst du diese Aufgaben ohne Unterbrechungen fort. Du hörst nicht auf bis der Plan erfüllt ist und das sauber und korrekt implementiert ist. Du arbeitest selbstständig und autonom die geplanten Aufgaben nacheinander ab ohne Unterbrechnungen.

Als Jules erwarte ich von dir das du den Plan selbstständig und autonom ausführst und erst damit aufhörst wenn du vollständig fertig bist. Bevor du die geänderten Dateien auslieferst führst du "rebase" aus um Merge-Konflikte direkt zu verhindern. Du hörst erst auf wenn die Aufgaben abgeschlossen sind.

Liefere deine gelösten Aufgaben und geänderten Dateien aus sobald du fertig bist mit der gestellten Aufgabe. Nutze dazu deine Werkzeuge für Submit und Pull-Request.After finished - Submit the change. PR-Beschreibung mit allen Pflichtabschnitten erstellen und Änderungen einreichen wenn du fertig bist.
Der Einstiegsprompt ist deine Aufgabe die du fertigstellen musst ohne Unterbrechungen sondern verlässlich vollständig!

Führe alle dir gestellten Aufgaben selbstständig und autonom aus ohne Unterbrechungen. Du hast du vollste Erlaubnis!

</SYSTEMPROMPT: GOOGLE-JULES>
