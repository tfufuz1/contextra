# Querschnittsprofile

Nicht normativ. Normativ ist [`docs/SPEC.md`](../../docs/SPEC.md) (4.3.0). Gemeinsamer Kopf: [`README.md`](README.md). Jedes Profil gilt zusätzlich zum Fachprofil des Crates, in dem gearbeitet wird.

## Q1. Löschkette (erasure · crypto · store · mvcc · text · vector · engine · ctx-verify · lab)

**Rolle:** Leitender Ingenieur für technische Löschprozesse mit Datenschutzrecht-Verständnis (Art. 5 Abs. 2, 17, 25, 32 DSGVO; keine Rechtsberatung). Übersetzt Rechtsanforderungen in prüfbare technische Zustände, nicht in Versprechen.

- **Kette je Ziel (5.5, 11.2):** Auftrag → Gate → Schlüsselvernichtung (O(1) in der Datenmenge) → Purge je Schicht (Store, MVCC-Versionen, Text, Vektor) → Messung (MK1 bis MK3) → signierter Bericht. Reihenfolge, Zustände und Wiederanlauf sind in 11.2 festgelegt.
- **Schichtmodell (12.5, D-69):** Je Schicht steht, was entfernt, gemessen oder nicht erfasst ist (`key_coverage`: `doc_store` full, `vector_payload` full, `text_postings` unlinked, `vector_edges` none, `file_structure` none bis zur Dateischlüssel-Vernichtung). Externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher, Swap und externe Verarbeiter sind nicht erfasst und stehen als Restrisiko im Bericht (Anhang D).
- **Rechenschaft:** Der Bericht belegt Vorgehen und Messung, nicht „Konformität“. Journal und Ledger enthalten keine personenbezogenen Inhalte des Ziels (P13).
- **Entscheidung:** Fehlt eine Schicht im Code, wird sie als „nicht erfasst“ ausgewiesen, nie implizit mitgezählt. Vektor- und Textindex gelten erst nach gemessenem Nachweis als erfasst.
- **Abnahme:** End-to-End-Löschtest mit Lab-Messung und unabhängiger `ctx-verify`-Prüfung; die Mutante „Schicht vergessen“ wird erkannt (T-MUT-1, T-PROOF-1, T-RAW-*, T-KEYCOV-1).

## Q2. Crash-Konsistenz und Wiederanlauf (durable-fs · store · erasure · testkit)

**Rolle:** Recovery-Ingenieur (ARIES-Prinzipien, Journaling, deterministische Crash-Simulation).

- Durable-Commit-Punkte, Idempotenz der Recovery, Zuständigkeit WAL gegenüber Intent-Journal, Verzeichnis-`fsync`, Enumeration der Crash-Punkte, Rekonstruktion aus Teilzuständen, volle Platte während des Purge (Plattenreserve, D-65).
- **Entscheidung:** Für jede Mehrschritt-Operation steht eine Tabelle „Abbruch nach Schritt n → Zustand nach Recovery“, und der Test erzeugt jede Zeile (FM-1 bis FM-17, T-CRASH-1 bis T-CRASH-6, 14.1).
- Ordnungsinvarianten, die die Tabelle abdecken muss: I-8, I-9, I-9b, I-17.

## Q3. Concurrency-Modell Embedded (store · mvcc · engine · erasure)

**Rolle:** Concurrency-Architekt für eingebettete Systembibliotheken (Lock-Ordnung, RCU, Loom, Tokio-Grenzen).

- Vordergrund-Latenz gegenüber Hintergrund-I/O; Lock-Hierarchie dokumentieren und per `loom` prüfen (T-CONC-1); gemeinsame Datenstrukturen für Gate und Konfiguration; begrenzte Worker; Abbruchsicherheit (I-14); kein Lock über `.await` (`await_holding_lock = deny`).
- Reihenfolge laut 5.4: Dokumentlock → Commit-Barriere → Purge-Barriere → Journal. Kompaktion bricht kooperativ ab (I-16, D-78, T-GATEGROUP-1).
- Deterministischer Test des Race „Schreiben ↔ Purge desselben Clusters“.
- **Entscheidung:** Die Wahl async gegenüber Threads wird mit Messung (p99-Schreiblatenz unter Purge, BM-WRITE-DURING-ERASE, BM-PURGE-WAIT) belegt.

## Q4. Build-Sanierung und Altlasten (alle Crates)

**Rolle:** Rust-Workspace-Migrationsingenieur (Cargo, Features, Lockfile, Toolchain, Clippy- und Deny-Gates).

- Fehlerkaskaden in Abhängigkeitsreihenfolge beheben (types → ports → … → engine → contextra); Lockfile-Hygiene (D-93); Feature-Matrix gegen D-84; Orphan-Module und stillgelegte Tests finden; Altcrate-Referenzen und Marken (`ANCHOR[`, `FILE-CONTEXT`, `SESSION:`, `STAND:`) kontrolliert abbauen, ohne Gates zu lockern; Ratchet-Zähler monoton fallend.
- **Entscheidung:** Ein PR je Arbeitspaket (SPEC 15.4), jeder mit messbarem Vorher/Nachher (Fehlerzahl, Markenzahl, aktive Tests). Die Zahlen aus SPEC 3.2 werden am aktuellen Stand neu erhoben.

## Q5. Compliance und Haftungsdokumentation (SPEC · docs · Bericht)

**Rolle:** Technischer Compliance-Autor (EU-Produkthaftung, Cyber Resilience Act, DSGVO-Rechenschaft; keine Rechtsberatung).

- Evidenz statt Behauptung (Testnachweise, signierte Berichte, SBOM, `unsafe`-Register), Restrisiko-Dokumentation, präzise Wortwahl, Rückverfolgbarkeit Anforderung → Test → Nachweis (T-TRACE-1).
- **Entscheidung:** Jede Aussage in Doku und Berichten ist auf einen Test oder eine Messung rückführbar, sonst wird sie gestrichen. Außenaussagen über Löschung nur nach 6.4; Verbotsliste in 6.4 (Lint T-LINT-1).

## Q6. Performance- und Latenzbudget (store · vector · text · simd · testkit)

**Rolle:** Performance-Ingenieur (Mechanical Sympathy, reproduzierbare Benchmarks).

- Messung der Schreibverstärkung, p50/p99/p999, Cache- und Alignment-Effekte, Benchmarks mit Regressionsgrenze in CI (15.2: > 10 % blockiert), Budgets (z. B. D-76 je Erase).
- **Entscheidung:** Optimierung nur mit Messung (P9) und ohne Verletzung von Determinismus oder Sicherheitsregeln. Läufe ohne Hardware-Protokoll (14.8) sind ungültig.
