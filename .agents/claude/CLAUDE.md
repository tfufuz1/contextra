# Contextra – Betriebsanleitung für Claude und Antigravity

Nicht normativ. Maßgeblich ist [`docs/SPEC.md`](../../docs/SPEC.md) (Version 4.2.0). Diese Datei fasst die Regeln für die Arbeit am Code zusammen und verweist auf die SPEC.

## 1. Arbeitsweise

- Ein PR bearbeitet ein Arbeitspaket aus SPEC 15.4. Größere Änderungen werden geteilt.
- Rot vor Grün: Vor der Implementierung steht ein fehlschlagender Test mit SPEC-ID (D-44, P10).
- Keine Annahmen über APIs, Dateipfade oder Symbole aus dem Gedächtnis. Vor der Verwendung im Code nachsehen.
- Kleinste sinnvolle Änderung. Keine Abstraktion ohne Kernkonsumenten (P8).
- Protokolle für Gates gehören in CI-Artefakte oder das Evidenzbündel, nicht in den Quellbaum (D-98).

## 2. Harte Regeln, kurz

1. **Ringe.** Abhängigkeiten nur nach unten (SPEC 5.2). Innerhalb Ring 2 gilt die Matrix SPEC 5.3.
2. **Keine Panics.** Kein `unwrap`, `expect`, `panic!`, `unreachable!` und kein ungeprüftes Slicing im Produktionscode (P6).
3. **Determinismus.** Zeit, Zufall und IDs nur über `Clock`, `Rng`, `IdGen` (P7).
4. **WAL-Reihenfolge.** Frames, Gruppen-Commit, `sync_data`, Memtable, Sichtbarkeit (I-9).
5. **Ein I/O-Weg.** Nur `Vfs`. Kein `std::fs`, kein `tokio::fs`, kein `mmap` (D-04, D-94).
6. **Zwei `unsafe`-Inseln.** `contextra-sys` und `contextra-simd`, jeweils mit `// SAFETY:`. Alle anderen Crates: `#![forbid(unsafe_code)]` (D-29).
7. **Mandanten.** Jede Funktion benötigt einen `TenantContext`. Rohspeicher nur über `TenantScopedStorage` (D-07, AR-6).
8. **Keine Schalter für Sicherheit.** Verschlüsselung und WAL-Integrität sind immer aktiv (D-05, HR-13). Argon2id mit den Grenzen aus D-06.
9. **Messen statt behaupten.** Restbestände stammen aus Zeugen mit Positivkontrolle (P1, D-72).
10. **Fail-closed.** Unbekannte Zustände und Krypto-Fehler führen zu Abbruch oder `Pending`, nie zu stillem Weiterlaufen (P2).
11. **Ein Typ je Konzept.** Domänentypen in `contextra-types`, Kryptografie in `contextra-crypto` (SPEC 8.1, 8.6).
12. **Gates.** Kein Phasenwechsel ohne bestandenes Gate (SPEC 15.3).

## 3. Verbote im Code und in Dokumenten

- Löschaussagen außerhalb SPEC 6.4 (Wortlaut-Lint, T-LINT-1).
- Agenten-, Sitzungs- und Zeitstempelmarken im Quelltext (D-91).
- Absolute Entwicklerpfade in Dateien oder Dokumenten (D-91).
- Neue Abhängigkeiten, `unsafe`-Blöcke oder Features ohne belegenden Test (HR-12).
- Schalter, die Sicherheit abschalten, auch über Umgebungsvariablen (HR-13).

## 4. Prüfen

Befehle: `docs/COMPILATION.md`. Vor dem Abschluss eines Arbeitspakets sind alle Gates des Pakets grün zu melden. Ist ein Gate rot, ist die Ursache zu beheben. Gates, Lints, Baselines und Toolchain-Pins werden nicht geändert, um Rot zu beseitigen (`.agents/jules/PREAMBLE.md`, Regel 4).
