# Contextra – Kontext für Jules-Sitzungen

Nicht normativ. Maßgeblich ist [`docs/SPEC.md`](../../docs/SPEC.md), Version 4.2.0.

## Ladeordnung

1. `.agents/jules/SESSION_BOOTSTRAP.md`: Ablauf einer Sitzung.
2. `.agents/jules/PREAMBLE.md`: Grundregeln.
3. `AGENTS.md` im Wurzelverzeichnis: Ist-Zustand und harte Regeln.
4. `docs/SPEC.md`: Soll-Zustand, nur die Abschnitte der Aufgabe.
5. `crates/<crate>/AGENTS.md`: Hinweise je Crate.

## Projektstand

- Contextra ist eine eingebettete Rust-Bibliothek für lokale Speicherung und nachweisbares Löschen in Agenten- und RAG-Systemen.
- Der Workspace baut derzeit nicht vollständig (SPEC F-32). Der Löschkern `contextra-erasure` ist leer (F-33).
- Die Arbeit folgt den Arbeitspaketen in SPEC 15.4. Die Reihenfolge der Gates ist A, B, G0, C, D, F, E, G, H.

## Crate-Topologie

Die verbindliche Ringzuordnung und die erlaubten Abhängigkeiten stehen in SPEC 5.1, 5.2 und 5.3. Frühere Quellen, die als Architekturgrundlage dienten (`WORKING_STATE.md`, `capabilities.toml`), sind nicht maßgeblich. Die Ring- und Abhängigkeitsprüfung erfolgt durch `contextra-arch-tests` (T-ARCH-3, T-ARCH-5, T-ARCH-6).

## Was Jules nicht tut

- Keine Änderung an Verschlüsselung, Integritätsprüfung, Löschlogik oder Mandantenprüfung ohne Arbeitspaket und Freigabe.
- Keine neuen Abhängigkeiten, `unsafe`-Blöcke oder Features ohne Begründung im PR (HR-12).
- Keine Löschaussagen außerhalb SPEC 6.4.
