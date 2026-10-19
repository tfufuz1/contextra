# ADR-N30: GitHub als Steuerzentrale und git-lesbarer Agentenzustand

* **Status:** Accepted
* **Datum:** 2026-10-19
* **Autoren:** Principal Architect (Contextra)
* **Betroffene Komponenten:** Governance, GitHub Integration, Agenten-Harness (`.jules/`)
* **Spezifikationsreferenzen:** Spezifikation v15 Teil G (Governance & CI), `.jules/ISSUE_AUTOMATION_POLICY.md`

---

## 1. Ist-Zustand & Befund (mit Pfad- und Zeilennachweisen)

Eine präzise Bestandsaufnahme im Repository zeigt eine Entkopplung zwischen der Projektsteuerung und den nativen GitHub-Steuerungsfunktionen:

1. **GitHub-Steuerungsfunktionen ungenutzt:**
   - **Milestones & Issues:** Es existieren 0 Milestones und 0 offene/geschlossene Issues im Repository.
   - **Wiki:** Das GitHub-Wiki ist nicht aktiviert und findet keine Anwendung im Entwicklungs- und Governance-Prozess.
   - **Projektstand-Split:** Der operative Entwicklungszustand, Task-Fortschritte und Protokolle sind unstrukturiert in Flachdateien wie `WORKING_STATE.md` (10.923 Zeilen/Bytes), `LOG.md` (9.062 Bytes) sowie `.jules/JULES_LOG.md` (10.916 Bytes) und `.jules/JULES_LOG_2.md` (8.554 Bytes) verstreut.

2. **Einschränkung der Issue-Automatisierung:**
   - In `.jules/ISSUE_AUTOMATION_POLICY.md:1–45` ist festgelegt, dass Issues ausschließlich vollautomatisch durch das System bei einem fehlerhaften `main`-Branch ("main rot") erzeugt werden dürfen. Manuell von Entwicklern oder Architekten angelegte Issues sind laut bisheriger Policy untersagt.

3. **REST-API Rate-Limits in VM-Umgebungen:**
   - Unauthentifizierte Anfragen an die GitHub-REST/GraphQL-API sind streng auf 60 Requests pro Stunde und IP-Adresse limitiert (`https://api.github.com`). Autonome VM-Instanzen (wie Jules) stoßen bei intensiven API-Abfragen schnell an dieses Limit.
   - Git-Operationen (`git fetch`, `git checkout`, `git show`) nutzen hingegen das Git-Protokoll bzw. authentifizierten/gecachten Transport und unterliegen nicht dem 60-Req/h-API-Rate-Limit.

---

## 2. Entscheidung & Soll-Zustand

1. **GitHub Issues & Issue-Forms als primäre Task-Eingabe:**
   - GitHub Issues ersetzen die lokalen Task-Karten (`.jules/tasks/T-*.toml`) als primäre Aufgaben-Schnittstelle.
   - Jede Aufgabe wird über vordefinierte GitHub Issue-Forms (`.github/ISSUE_TEMPLATE/*.yml`) strukturiert erfasst.

2. **Milestones als Roadmap-Phasen:**
   - GitHub Milestones werden als strukturierte Container für Entwicklungsphasen und Releases (z. B. "Phase 1b: Strangler Facade", "Phase 2: Sync Core Refactoring") eingesetzt.

3. **Standardisiertes Labels-Schema:**
   Anwendung eines verbindlichen Label-Katalogs zur maschinellen Steuerung:
   - `jules:ready`: Task ist vollständig spezifiziert und für den Agenten freigegeben.
   - `jules:in-progress`: Task wird aktuell von einem Agenten bearbeitet.
   - `jules:needs-human`: Agent benötigt menschliche Entscheidung/Intervention.
   - `protected-change`: Task betrifft geschützte Pfade (`protected-paths.toml`).
   - `blocked`: Task ist durch externe Abhängigkeiten blockiert.
   - `type:*`: `type:bug`, `type:feature`, `type:refactor`, `type:docs`, `type:governance`.
   - `tier:*`: `tier:1`, `tier:2`, `tier:3` (Orientierung an Task-Größe und -Tiefe).
   - `risk:*`: `risk:sec`, `risk:crash`, `risk:simd`, `risk:utf8`, `risk:fusion`, `risk:concurrency`, `risk:calib`, `risk:none`.
   - `crate:*`: Betroffenes Crate (z. B. `crate:contextra-store`, `crate:contextra-engine`).

4. **Kein GitHub-Wiki:**
   - Das GitHub-Wiki wird explizit **nicht** genutzt, da Wiki-Inhalte außerhalb des Pull-Request-Reviews, der Qualitäts-Gates und der Git-Historie liegen. Alle Dokumentationen verbleiben im Haupt-Repository unter `docs/`.

5. **Git-lesbarer Agentenzustand auf Branch `jules-state`:**
   - Um das API-Rate-Limit (60 Req/h) zu umgehen, wird ein dedizierter Orphan-Branch `jules-state` geführt, der ausschließlich die Datei `STATE.md` enthält.
   - Agenten lesen und aktualisieren ihren Ausführungszustand direkt via Git-Befehlen (`git fetch origin jules-state`, `git show origin/jules-state:STATE.md`).

6. **Lockerung der `.jules/ISSUE_AUTOMATION_POLICY.md`:**
   - Die Policy wird angepasst: Manuell von Menschen angelegte Issues sind ausdrücklich erlaubt. Automatisch erstellte Issues verbleiben weiterhin streng auf deduplizierte Meldungen bei "main rot" beschränkt.

---

## 3. Konsequenzen

* **Positiv:**
  - Lückenlose Transparenz der Aufgaben im GitHub-UI für Entwickler und Maintainer.
  - Vermeidung von API-Rate-Limit-Engpässen durch Zustandshaltung im Git-Branch `jules-state`.
  - Vermeidung von Dokumentations-Silos außerhalb von Git (kein Wiki).
* **Negativ:**
  - Synchronisationsaufwand für den Branch `jules-state` bei parallelen Agenten-Sessions.

---

## 4. Verworfene Alternativen

* **Verbleib bei lokalen Flachdateien (`WORKING_STATE.md`, `.jules/tasks/`):**
  Verworfen wegen fehlender Transparenz auf GitHub und Unübersichtlichkeit bei wachsender Session-Anzahl.
* **Ausschließliche Nutzung der GitHub REST-API für Zustandstracking:**
  Verworfen wegen des strikten API-Limits von 60 Anfragen/Stunde in unauthentifizierten VM-Kontexten.

---

## 5. Migrationsschritte

1. Anlegen der GitHub Issue-Forms unter `.github/ISSUE_TEMPLATE/` für Task-Eingaben.
2. Erstellen des Orphan-Branches `jules-state` mit initialer `STATE.md`.
3. Aktualisierung der `.jules/ISSUE_AUTOMATION_POLICY.md` zur Freigabe manueller Issue-Erstellung.
4. Archivierung/Löschung veralteter lokaler Fortschrittsdateien (`WORKING_STATE.md`, `LOG.md`).
