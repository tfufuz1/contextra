# Governance Rule: Kanonische Tag-Taxonomie & Dezentrales Inline-Kommentierungssystem (`rules/tag_taxonomy.md`)

> **Status:** Normativ · **Stand:** Oktober 2026 · **Referenz:** ADR-028, AGENTS.md §7

## 1. Zweck & Leitprinzip

Das dezentrale Inline-Kommentierungssystem verankert Kontext, Invarianten, technische Schulden und Review-Nachweise direkt im Quellcode am Ort des Geschehens.
Gemäß **ADR-028** ist `WORKING_STATE.md` eine reine, voll-generierte Projektion dieser Inline-Tags. Alle Zeitstempel sind sekundengenau in UTC (`ISO-8601`), und alle Tags tragen kryptografisch deterministische Hash-IDs sowie Session-Tokens zur Verhinderung von Kollisionen bei parallelen Agenten-Sitzungen.

---

## 2. Kanonische Tag-Formate

Jeder im Code platzierte Tag muss exakt einer der folgenden vier Formen entsprechen:

### A. Befund- & Schulden-Tags (`AI-TAG`)

```rust
// AI-TAG[KATEGORIE][SEVERITY] Kurzbeschreibung (ID: AGT-<CRATE>-<8-hex>) (TS: <ISO-8601-UTC>) (SESSION: <HASH>)
// BEFUND: Detaillierte Analyse des Ist-Zustands.
// RISIKO: Was passiert, wenn dies unbehoben bleibt.
// EMPFEHLUNG: Konkrete Handlungsempfehlung.
```

- **Kategorien:** `CORRECTNESS`, `SECURITY`, `PANIC-SAFETY`, `CONCURRENCY`, `PERF`, `SMELL`, `HARDENING`, `DOC-DRIFT`, `CONVENTION-DRIFT`, `INVENTORY-DRIFT`, `SLEEP`
- **Schweregrade (Severity):**
  - `CRITICAL` / `BLOCKER`: CI bricht ab. In Implementierungs-Sessions sofort beheben.
  - `MAJOR`: Warnung in CI.
  - `MINOR`: Getrackt in `WORKING_STATE.md`.
- **Ablaufdatum bei Audit-Schulden:**
  Für befristete Audit-Befunde ist der Parameter `until=YYYY-MM-DD` zulässig.
  ```rust
  // AI-TAG[SMELL][MINOR] TODO(audit-C-1): Beschreibung (ID: AGT-STORE-bae66245) (TS: 2026-09-10T19:14:58Z) (SESSION: 21a8d3e8) until=2026-12-15
  ```
  Ab dem Stichtag `2026-12-31` (`TAG_UNTIL_MANDATORY_CUTOFF_DATE`) ist `until=` für offene Audit-Tags verpflichtend.
- **Auflösung (Schließen) eines Befunds:**
  ```rust
  // RESOLVED: AGT-<CRATE>-<8-hex> — <Fix-Beschreibung> (TS: <ISO-8601-UTC>)
  ```
  oder inline:
  ```rust
  // AI-TAG[KATEGORIE][SEVERITY][RESOLVED] ... (ID: AGT-...) (TS: ...) (SESSION: ...)
  ```

### B. Aufgaben- & Implementierungs-Anker (`ANCHOR`)

```rust
// ANCHOR[TYP:ID] STATUS:OPEN (TS: <ISO-8601-UTC>) (SESSION: <HASH>)
// AUFGABE : Was zu implementieren ist.
// GATE    : cargo test <crate> --test <testname>
```

- **Typen:** `TEST`, `PERF`, `INTEGRATION`, `REFACTOR`, `GATE`, `IMPL`
- **Status-Werte:** `OPEN`, `IN_PROGRESS`, `PARTIAL`, `DONE`, `RESOLVED`
- **Regel für `STATUS:DONE`:** Ein Agent darf `STATUS:DONE` **nicht** im Alleingang setzen. Jede `DONE`-Markierung erfordert unabhängige `REVIEW-PASS`-Bestätigungen.

### C. Mehrfach-Session-Review (`REVIEW-PASS`)

```rust
// REVIEW-PASS[N/M] STATUS:PASS|FAIL|CONDITIONAL (TS: <ISO-8601-UTC>) (SESSION: <HASH>)
// PRÜFER-KONTEXT: FRESH
```

- **Quorum-Regel (Gate 8):**
  - **Standard-Code:** Mindestens **2** unabhängige `REVIEW-PASS`-Einträge mit unterschiedlichen `SESSION:`-Hashes.
  - **Kritischer Code (Unsafe-Inseln & Security):** Mindestens **3** unabhängige `REVIEW-PASS`-Einträge für Dateien in Unsafe-Inseln (`contextra-vector/src/distance.rs`, `diskann.rs`, `persistence.rs`, `contextra-crypto/src/anti_tamper.rs`) oder Tags mit Typ `SECURITY`.
- Wird technisch durch `cargo xtask check-review-coverage` erzwungen.

### D. Datei-Kontext-Header (`FILE-CONTEXT`)

Jede `.rs`-Datei > 50 Zeilen mit Invarianten oder bekannten Fallstricken erfordert am Dateikopf:

```rust
// FILE-CONTEXT
// STAND: <ISO-8601-UTC> (SESSION: <HASH>)
// ZWECK: <Ein Satz — was diese Datei tut>
// INVARIANTEN: <Was bei jeder Änderung gelten MUSS>
// NICHT-OFFENSICHTLICH: <Architektur-Entscheidungen, die ohne dieses Wissen zu Fehlern führen>
// SIEHE AUCH: <Pfade zu relevanten ADRs oder rules/*.md>
// AGENT-NOTIZ: <Optional, max. 1 Satz dezentraler Kommunikationskanal>
```

---

## 3. Zeitstempel-, Datums- und ID-Spezifikation

1. **Zeitstempel-Format:**
   - Sekundengenaue UTC-Zeit nach ISO-8601: `YYYY-MM-DDTHH:MM:SSZ` (z. B. `2026-10-14T17:30:00Z`).
   - Reines Datumsformat `YYYY-MM-DD` ist ausschließlich für `until=` und Bestands-Header vor 2026-08-29 zulässig.
2. **Session-Token:**
   - Pflichtfeld `(SESSION: <8-hex-hash>)` bei allen neu angelegten oder modifizierten Tags.
3. **Deterministische Hash-ID:**
   - Format: `AGT-<CRATE-SHORT>-<8-hex>`
   - Berechnung: `sha256(crate + pfad + zeile + ts)[..8]`
   ```bash
   printf '%s' "${crate}crates/${crate}/src/<datei>.rs<zeile>${TS}" | sha256sum | head -c 8
   ```
   - Bestehende historische IDs (`AGT-<CRATE>-NNN`) genießen Bestandsschutz.

---

## 4. Technische Durchsetzung via `xtask`

| Befehl | Zweck | Invariante |
|---|---|---|
| `cargo xtask validate-tags` | Prüft Syntax, ID- und Timestamp-Gültigkeit aller Tags | ADR-028 |
| `cargo xtask check-stale-tags [--threshold <tage>] [--strict]` | Erkennt abgelaufene `until=`-Tags und veraltete Audit-Schulden | ADR-028 |
| `cargo xtask tag-health` | Erkennt Code-Änderungen an Dateien nach Tag-Platzierung (`FileModifiedAfterTag`, `LocalHotspot`) | ADR-028 |
| `cargo xtask check-review-coverage` | Erzwingt 2 bzw. 3 unabhängige Review-Passes vor `STATUS:DONE` | ADR-028 §4 |
| `cargo xtask sync-docs [--check]` | Generiert `WORKING_STATE.md` deterministisch aus Inline-Tags | ADR-028 §1 |

---

## 5. Häufige Fehler & Anti-Patterns

- ❌ **Zeitstempel ohne `Z` oder ohne Sekunden:** `TS: 2026-10-14` statt `TS: 2026-10-14T12:00:00Z`.
- ❌ **Fehlendes `SESSION:`-Token:** Neu geschriebener Tag ohne Session-Hash.
- ❌ **Eigenmächtiges `STATUS:DONE`:** Setzen von `DONE` an einem `ANCHOR` ohne die erforderliche Anzahl unabhängiger `REVIEW-PASS`-Zeilen.
- ❌ **Manuelles Editieren von `WORKING_STATE.md`:** `WORKING_STATE.md` darf ausschließlich via `cargo xtask sync-docs` aktualisiert werden.
