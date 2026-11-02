# Beitragen zu Contextra

Dieses Dokument fasst die Regeln für Beiträge zusammen. Maßgeblich ist [`docs/SPEC.md`](SPEC.md) (Version 4.2.0). Bei Widerspruch gilt die SPEC.

## 1. Voraussetzungen

- Rust-Toolchain laut `rust-toolchain.toml`. Zum Sollwert und zum Pin siehe [`COMPILATION.md`](COMPILATION.md).
- Pflichtlektüre: SPEC Kapitel 0 (Konventionen), 2 (Leitprinzipien), 6.3 (harte Regeln), 13 (Invarianten), 15 (Tests und Gates).

## 2. Arbeitsweise

1. **Arbeitspaket.** Jede Änderung gehört zu einem Arbeitspaket aus SPEC 15.4 (`WP-x.y`). Ein PR bearbeitet ein Arbeitspaket. Querschnittsänderungen werden getrennt eingereicht.
2. **Rot vor Grün (D-44, P10).** Vor der Implementierung steht ein fehlschlagender Test mit der betroffenen SPEC-ID. Ohne Test keine Entscheidung.
3. **Rückverfolgbarkeit (T-TRACE-1).** Tests tragen `#[ctx_test(id = "T-…")]`, Benchmarks `#[ctx_bench(id = "BM-…")]`. Jede referenzierte ID muss existieren und mindestens einen Test oder Benchmark haben.
4. **Evidenz.** Protokolle eines Arbeitspakets werden als CI-Artefakt oder im Release-Evidenzbündel abgelegt, nicht im Quellbaum (D-98, SPEC 17.1).
5. **Lockfile (D-93).** `Cargo.lock` wird im selben Commit wie jede Manifest-Änderung erneuert. Prüfungen laufen mit `--locked`.

## 3. Vor jedem PR lokal

Die Befehle stehen in [`COMPILATION.md`](COMPILATION.md). Mindestens:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p contextra-arch-tests --locked
```

## 4. Pflichtangaben im PR (HR-12)

- **Neuer `unsafe`-Block:** Begründung, `// SAFETY:`-Kommentar mit Vorbedingungen, Test-ID. `unsafe` ist nur in `contextra-sys` und `contextra-simd` erlaubt (D-29).
- **Neue Abhängigkeit:** Prüfung gegen die Matrix SPEC 5.3, `cargo deny check`, Nachweis, dass keine Netzwerk-Bibliothek in den Produktionsbaum gelangt (`cargo tree -e normal`, HR-1).
- **Neues Feature:** Deklaration in `[features]`, belegender Test. Kein Feature darf ein Sicherheitsmerkmal abschalten (P5, HR-13).
- **Betroffene SPEC-IDs** (D-, I-, FM-, T-, BM-).

## 5. Verbote

Vollständig in SPEC 2, 6.3 und 6.4. Kurzfassung:

- Panics und verschluckte Fehler im Produktionscode (P6).
- Direkte Zugriffe auf Dateisystem, Systemuhr oder Zufall außerhalb der Ports (P7, D-04).
- Netzwerk-, Telemetrie- und Escrow-Code (HR-1 bis HR-3).
- Schalter, die Verschlüsselung, Integrität, Löschmessung oder Mandantenprüfung abschalten (HR-13).
- `mmap` (D-94) und `block_on` in Bibliothekscode (D-03).
- Agenten-, Sitzungs- und Zeitstempelmarken sowie absolute Pfade im Quelltext und in Dokumenten (D-91).
- Löschaussagen außerhalb SPEC 6.4. Der Wortlaut-Lint (T-LINT-1) prüft Quelltext, Rustdoc, README und Doku.

## 6. Dokumentation

- Normativ ist nur die SPEC. Abgeleitete Dokumente werden nur nach einer SPEC-Änderung mit Eintrag in Anhang H angepasst.
- Abgeleitete Dokumente nennen den SPEC-Abschnitt, auf den sie sich stützen. Ändert sich eine Tatsache, werden alle betroffenen Dokumente im selben PR angepasst (D-98).

## 7. Review

Reviewende prüfen mindestens: vorhandener roter Test, Einhaltung der Ringe und der Matrix, `Cargo.lock` im Commit, keine Verbote aus Abschnitt 5, Test-IDs, Lint-Freiheit.
