# SPEC: `contextra-testkit` (Ring Tooling, Klasse T)

> **Normativer Status:** Verbindliche Spezifikation gemäß SPEC v4.0 §2, §3, §4, §5.4 (Klasse T Werkzeug / reine Testhilfe).
> **Sicherheits-Invariante (INV-EMBED-1):** `#![forbid(unsafe_code)]` im gesamten Produktions- und Testcode; darf ausschließlich als `[dev-dependencies]` in Produktions-Crates eingebunden werden.
> **Ring-Zugehörigkeit:** Ring Tooling (außerhalb der numerischen Ringe 0 bis 4).

---

## 1. Zweck & Abgrenzung

`contextra-testkit` stellt Test-Doubles (`ManualClock`, `FaultVfs`, In-Memory-Stores, Mocks) und wiederverwendbare generische Port-Konformitätssuiten für Contextra-Ports bereit.

### Ziele
- **Deterministische Testumgebungen:** Volle Kontrolle über Zeit (`ManualClock`), Zufall (`SeededRng`) und I/O-Fehler (`FaultVfs`) ohne Flakiness.
- **Contract Verification (Konformität):** Standardisierte Prüfsuiten für Adapter gegen Port-Spezifikationen (`Vfs`, `Clock`, `Rng`, `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `MetricsSink`).
- **Fail-Closed & Zero-Panic:** Fehlererkennung liefert strukturierte `ConformanceFailure`-Objekte anstelle unkontrollierter Abbrüche.
- **Klasse-T-Isolation:** Verwendbar in Klasse B/C; strikt verboten in Klasse A (E-12, B2).

### Nicht-Ziele
- Keine Produktionsimplementierung: Doubles und Mocks dürfen niemals in Release-Code oder Standard-Verdrahtungen verwendet werden (INV-EMBED-1).
- Keine Löschdomänen-Orchestrierung: Juristische Löschsagas und Deletion-Proofs gehören exklusiv in `contextra-erasure` (§6). Testkit stellt lediglich `mint_witness_for_tests` hinter dem Feature `erasure-conformance` bereit.

---

## 2. Invarianten (DbC)

### INV-ORACLE-1: Spezifikationskonforme Orakel
Test-Doubles verhalten sich exakt wie die Schnittstellenspezifikation der Ports vorschreibt, unabhängig von der konkreten internen Implementierung produktiver Speicherkerne.

### INV-FAULT-DET-1: Deterministische Crash-Matrix
`FaultVfs` bildet Fehlerszenarien deterministisch über konfigurierbare Zähler ab:
- `fail_writes_after`: Abweisung von Schreibversuchen ab Erreichen des Schwellenwerts mit `io::ErrorKind::WriteZero`.
- `fail_syncs_after`: Simulation fehlschlagender `fsync`-Befehle mit I/O-Fehler.
- `fail_reads_after`: Simulation fehlerhafter Plattenblöcke mit `io::ErrorKind::UnexpectedEof`.
- `trigger_crash`: Sofortige Abweisung aller Lese-, Schreib- und Sync-Operationen mit `io::ErrorKind::BrokenPipe` bis `reset_counters`.

### INV-CLOCK-MONO-1: Monotone Zeitfortschreitung
`ManualClock` garantiert strikte Monotonie (`now_unix_nanos` fällt niemals ab) und lässt sich ausschließlich kontrolliert über `advance` oder `set` fortschreiben.

---

## 3. Fehlermodell (`conformance::failure`)

Vertragsabweichungen in Konformitätsprüfungen werden in `ConformanceFailure` erfasst:
- `suite_name`: Name der geprüften Port-Schnittstelle (z. B. `"Vfs"`, `"StorageEngine"`).
- `test_case`: Spezifischer Testfall (z. B. `"write_atomic_and_read"`).
- `details`: Präzise Ursache der Vertragsverletzung.
`ConformanceResult` ist definiert als `Result<(), ConformanceFailure>`.

---

## 4. Nebenläufigkeits- & Zustandsmodell

- **Thread-Safety:** `FaultVfs` und `ManualClock` sind `Send + Sync` und nutzen thread-sichere atomare Zähler ([`AtomicUsize`], [`AtomicBool`]) sowie geschützte Zustände via [`parking_lot::RwLock`].
- **Wait-Free Reads / Zero Locks across `.await`:** Keine Sperren werden über `.await`-Grenzen gehalten.
