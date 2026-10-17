# ADR-N23: F8 FC-Thompson Routing Feature-Gating bis zum Replay-Nachweis

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Feature-Conditioned Thompson Sampling (FC-Thompson in `contextra-router` und `contextra-adapt`) stellt ein stochastisches Multi-Armed-Bandit-Routing-Verfahren bereit, um dynamische Exploration und Exploitation bei der Modellauswahl und Intent-Klassifikation durchzuführen.
  In Standard-Retrieval-Anwendungsfällen erzeugt stochastisches Routing jedoch Nicht-Determinismus im Lese-Pfad, erschwert das Benchmarking und kann unberechenbare Latenz-Spitzen auslösen.
  Zur Wahrung der P28-Deterministik-Invariante und vorhersehbaren Systemverhaltens erfordert die Aktivierung dynamischer Bandit-Mechanismen im Standard-Profil einen vorherigen empirischen Nachweis im Offline-Replay (Gesamtspezifikation §F8).

## Entscheidung

1. **Entfernung von FC-Thompson aus den Standard-Crate-Features:**
   FC-Thompson-Sampling und RIE-Greedy-Personalisiung werden aus den Standard-Features (`default`) von `contextra-router` und `contextra-adapt` entfernt und hinter explizite Opt-in Feature-Flags (`bandit-routing`, `rie-greedy-personalization`) gestellt [V]. Standardmäßig kommt deterministisches Conformal Routing zum Einsatz.

2. **Verbindliche Kriterien & Schwellenwert für Re-Inklusion:**
   Eine Re-Aktivierung von FC-Thompson als Standard-Verfahren setzt einen empirischen Nachweis in Offline-Replay-Benchmarks (`contextra-bench`) voraus.
   * **Kriterium:** Statistisch signifikante Steigerung der Retrieval-Qualität (Precision@k, Recall@k, MAP oder Ranking-Scores) gegenüber statischem/conformalem Routing.
   * **Schwellenwert:** Mindestens **$+5\%$ relativer Qualtätsgewinn** auf standardisierten Referenz-Datensätzen bei zeitgleicher Einhaltung des p99-Latenzbudgets ($\le +3\%$ Latenzaufschlag) [S].

## Alternativen

### Option A: FC-Thompson standardmäßig aktiviert lassen
* **Vorteile:** Dynamische Selbstanpassung des Routings ab dem ersten Aufruf.
* **Nachteile:** Stochastischer Nicht-Determinismus; unvorhersehbare Latenzen; bricht P28-Deterministik für Bibliothekseinbettesteller.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Deterministischer Standard:** Anwender erhalten standardmäßig exakt reproduzierbare Routing-Ergebnisse [V].
* **Feature-Gated Flexibility:** Fortgeschrittene Anwender können FC-Thompson gezielt aktivieren [V].
* **Messbare Re-Inklusion:** Die Schwelle für eine künftige Aktivierung ist vorab eindeutig quantifiziert [S].

## Reversibilitäts-Bedingung

Sobald Offline-Replay-Messungen den Nachweis erbringen, dass FC-Thompson den Schwellenwert von $+5\%$ Qualitätsgewinn innerhalb des Latenzbudgets einhält, wird das Feature wieder für den Standard-Build freigegeben [S].
