# Contextra – Betriebsanleitung für Claude und Antigravity

Nicht normativ. Maßgeblich ist [`docs/SPEC.md`](../../docs/SPEC.md) (Version 4.3.0). Diese Datei fasst die Arbeitsweise zusammen und verweist auf die SPEC. Gemeinsame Regeln, Ringe und Fachprofil-Tabelle: [`AGENTS.md`](../../AGENTS.md).

## 1. Arbeitsweise

- Ein PR bearbeitet ein Arbeitspaket aus SPEC 15.4. Größere Änderungen werden geteilt.
- Rot vor Grün: Vor der Implementierung steht ein fehlschlagender Test mit SPEC-ID `#[ctx_test(id = "T-…")]` (D-44, P10, T-TRACE-1).
- Keine Annahmen über APIs, Dateipfade oder Symbole aus dem Gedächtnis. Vor der Verwendung nachsehen (`grep`, Compiler).
- Kleinste sinnvolle Änderung. Keine Abstraktion ohne Kernkonsumenten (P8).
- Befehle tatsächlich ausführen und ehrlich berichten; nicht Gelaufenes als „nicht geprüft“ kennzeichnen. Offene Annahmen benennen statt still entscheiden.
- Protokolle für Gates gehören in CI-Artefakte oder das Evidenzbündel, nicht in den Quellbaum (D-98).

## 2. Fachprofil wählen

Lies `crates/<crate>/AGENTS.md` des Crates, in dem du änderst: Es enthält Rolle, normative Anker, Verbote, Abnahme und Prüfbefehle. Berührt die Änderung mehrere Crates, lies zusätzlich das passende Profil in `.agents/experts/querschnitt.md` (Q1 Löschkette, Q2 Crash, Q3 Concurrency, Q4 Build, Q5 Compliance, Q6 Performance) und die Regeln in `.agents/rules/`.

## 3. Harte Regeln, kurz

1. **Ringe.** Abhängigkeiten nur nach unten (SPEC 5.2); innerhalb Ring 2 gilt die Matrix SPEC 5.3.
2. **Keine Panics.** Kein `unwrap`, `expect`, `panic!`, `unreachable!`, `let _ =` auf `Result`, kein ungeprüftes Indexieren im Produktionscode (P6).
3. **Determinismus.** Zeit, Zufall, IDs nur über `Clock`, `Rng`, `IdGen` (P7).
4. **Reihenfolge der Dauerhaftigkeit.** Tresor-Slot vor WAL-Frame, `F`-Slot vor dem ersten Block, Nullung vor `unlink` (I-9, I-9b, I-17).
5. **Ein I/O-Weg.** Nur `Vfs`. Kein `std::fs`, kein `tokio::fs`, kein `mmap` (D-04, D-94).
6. **Zwei `unsafe`-Inseln.** `contextra-sys` und `contextra-simd`, jeweils mit `// SAFETY:`; alle anderen Crates `#![forbid(unsafe_code)]` (D-29, AR-8).
7. **Mandanten.** Jede Funktion benötigt einen `TenantContext`; Rohspeicher nur über `TenantScopedStorage` (D-07, AR-6).
8. **Keine Schalter für Sicherheit.** Verschlüsselung und WAL-Integrität sind immer aktiv (D-05, HR-13); Argon2id mit den Grenzen aus D-06.
9. **Messen statt behaupten.** Restbestände stammen aus Zeugen mit Positivkontrolle (P1, D-72).
10. **Fail-closed.** Unbekannte Zustände und Krypto-Fehler führen zu Abbruch oder `Pending` (P2).
11. **Ein Typ je Konzept.** Domänentypen in `contextra-types`, Kryptografie in `contextra-crypto` (SPEC 8.1, 8.6).
12. **Gates.** Kein Phasenwechsel ohne bestandenes Gate (SPEC 15.3).

## 4. Verbote im Code und in Dokumenten

- Löschaussagen außerhalb SPEC 6.4 (`.agents/rules/wording.md`, T-LINT-1).
- Agenten-, Sitzungs- und Zeitstempelmarken im Quelltext; absolute Entwicklerpfade (D-91).
- Neue Abhängigkeiten, `unsafe`-Blöcke oder Features ohne belegenden Test (HR-12).
- Schalter, die Sicherheit abschalten, auch über Umgebungsvariablen (HR-13).

## 5. Prüfen

Befehle: `docs/COMPILATION.md` und die Prüfbefehle im Crate-Profil. Vor dem Abschluss eines Arbeitspakets sind alle Gates des Pakets grün zu melden. Ist ein Gate rot, ist die Ursache zu beheben. Gates, Lints, Baselines und Toolchain-Pins werden nicht geändert, um Rot zu beseitigen (`.agents/jules/PREAMBLE.md`, Abschnitt C, Punkt 4).
