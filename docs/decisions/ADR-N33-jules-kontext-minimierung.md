# ADR-N33: Jules-Kontext-Minimierung und Kontext-Archivierung

* **Status:** Accepted
* **Datum:** 2026-10-19
* **Autoren:** Principal Architect (Contextra)
* **Betroffene Komponenten:** Agenten-Harness (`.jules/`), Entwickler-Dokumentation (`docs/`)
* **Spezifikationsreferenzen:** Spezifikation v15 Teil G (Agenten-Harness)

---

## 1. Ist-Zustand & Befund (mit Pfad- und Zeilennachweisen)

Ein Audit der Anweisungs- und Kontextdateien im Verzeichnis `.jules/` sowie im Root-Verzeichnis zeigt eine erhebliche Redundanz und Fragmentierung der Prompts und Protokolle:

1. **Redundante Prompt- und Instruction-Dateien:**
   - In `.jules/` existieren mehrere überschneidende Anweisungsdateien:
     - `.jules/PREAMBLE.md` (801 Bytes)
     - `.jules/SESSION_BOOTSTRAP.md` (4.457 Bytes)
     - `.jules/JULES_CONTEXT.md` (5.284 Bytes)
     - `.jules/HARNESS_QUICKSTART.md` (2.397 Bytes)
   - Diese Dateien wiederholen identische Prinzipien (Zero-Panic, Unsafe-Inseln, Sync-Kern, Gate-Anforderungen) in unterschiedlicher Detaillierung, was bei LLM-Agenten zu Kontext-Blat und Verwirrung führt.

2. **Verstreute Transkript- und Protokoll-Dateien:**
   - Entwicklungs-Logs und Transkripte sind im Workspace verstreut:
     - `WORKING_STATE.md` (10.923 Bytes in Root)
     - `LOG.md` (9.062 Bytes in Root)
     - `.jules/JULES_LOG.md` (10.916 Bytes)
     - `.jules/JULES_LOG_2.md` (8.554 Bytes)

---

## 2. Entscheidung & Soll-Zustand

1. **Konsolidierung in `.jules/PLAYBOOK.md` (≤ 80 Zeilen):**
   - Die vier bisherigen Anweisungsdateien (`PREAMBLE.md`, `SESSION_BOOTSTRAP.md`, `JULES_CONTEXT.md`, `HARNESS_QUICKSTART.md`) werden ersatzlos gelöscht.
   - Es wird eine einzige, prägnante Anweisungsdatei `.jules/PLAYBOOK.md` mit strikter Längenbegrenzung auf **maximal 80 Zeilen** angelegt.

2. **Minimale Ladeliste für Agenten-Sessions:**
   Beim Aufruf/Start einer Jules-Session lädt der Agent **ausschließlich** folgende vier Quellen in seinen Kontext:
   1. `AGENTS.md` (Root-Betriebsanweisung)
   2. `SESSION_START.md` (Dynamische Session-Initialisierung)
   3. Das zugewiesene GitHub-Issue bzw. die Task-Karte (`.jules/tasks/T-*.toml`)
   4. Das spezifische Crate-`AGENTS.md` (falls im Ziel-Crate vorhanden)

3. **Archivierung der Transkript-Logs:**
   - Alle Entwicklungs-Logs (`WORKING_STATE.md`, `LOG.md`, `JULES_LOG.md`, `JULES_LOG_2.md`) werden aus dem Root-Verzeichnis und aus `.jules/` entfernt und geordnet nach `docs/archive/jules/` verschoben.

---

## 3. Konsequenzen

* **Positiv:**
  - Drastische Reduktion des Prompt-Token-Verbrauchs zum Session-Start.
  - Höhere Befolgungstreue der Agenten durch Vermeidung widersprüchlicher/redundanter Instruktionen.
  - Aufgeräumtes Root-Verzeichnis.
* **Negativ:**
  - Historische Logs liegen künftig im Archivpfad `docs/archive/jules/` und müssen dort gezielt nachgeschlagen werden.

---

## 4. Verworfene Alternativen

* **Beibehaltung der getrennten Dateien (`PREAMBLE.md`, `JULES_CONTEXT.md` u. a.):**
  Verworfen, da Redundanzen zu Verwirrung und unnötigen Token-Kosten führten.

---

## 5. Migrationsschritte

1. Erstellung von `.jules/PLAYBOOK.md` (≤ 80 Zeilen).
2. Löschung von `PREAMBLE.md`, `SESSION_BOOTSTRAP.md`, `JULES_CONTEXT.md`, `HARNESS_QUICKSTART.md`.
3. Verschiebung von `WORKING_STATE.md`, `LOG.md`, `JULES_LOG.md`, `JULES_LOG_2.md` nach `docs/archive/jules/`.
4. Anpassung der Boot-Injektionslogik im Harness (`xtask`).
