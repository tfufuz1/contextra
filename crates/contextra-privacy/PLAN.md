# contextra-privacy — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §3 und `docs/crates/contextra-privacy.md`. **Ausgangslage:** Das Crate enthält umfangreiche Egress-/Exfiltrations-Altlogik und 41 Nicht-Test-Panikmuster statt des eng definierten Klassen- und Validierungsfundaments.

1. Reduziere die Eigentümerschaft auf `DataClass`, `Validated<T>`, `EntityRecognizer`, `SurrogateVault`, Egress-Policy und `PrivacyError`; entferne doppelte Gateway-/Audit-/Cache-Konzepte oder verlagere sie zum Endpoint.
2. Implementiere K-Schreibschutz: nur `Validated<T>` darf K-Payloads erreichen; Recognizer liefert nachvollziehbare Treffer/Unsicherheit und fail-closed bei erzwungener Policy.
3. Modellieren P/D/E eindeutig: P nur via `SubjectRef`, D nur mit `LineageRef`, K ohne `SubjectRef`; Egress redigiert oder verweigert entsprechend der Policy.
4. Ersetze alle Produktions-Panikpfade mit `PrivacyError`; keine Rohinhalte in Fehlermeldung, Trace oder Audit.
5. Leere Defaults, entferne Tokio/Netz-/höhere Abhängigkeiten aus dem Kern, ergänze Property-Tests für Klassengrenzen und Redaction.

**Abnahme:** Validierungs-/PII-/Egress-Negativtests, Log-Leak-Test, Zero-Panic- und Dependency-Gates.
