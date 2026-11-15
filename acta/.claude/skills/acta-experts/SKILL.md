---
name: acta-experts
description: "Waehlt den passenden Acta-Fachexperten (Subagent) fuer eine Aufgabe und gibt ihm den richtigen Auftrag. Verwenden, wenn Arbeit an einem bestimmten Crate, an der Loeschkette, an Crash-Recovery, Concurrency, Gates, Compliance oder Performance ansteht und Spezialwissen noetig ist."
---

# Acta-Experten einsetzen

Die Expertenprofile stehen in `docs/EXPERTEN-PROMPTS.md` (Teil A gemeinsamer Kopf, Teil B je Crate, Teil C Querschnitt). Als Subagenten liegen sie in `.claude/agents/` (39 Dateien, erzeugt aus dem Dokument; bei Aenderung des Dokuments neu erzeugen, nicht von Hand auseinanderlaufen lassen).

## Einschraenkung: nur Haiku
Sub-Agenten duerfen nur mit `model: haiku` gestartet werden (CLAUDE.md, Arbeitsregel 1). Alle Experten-Dateien tragen `model: haiku`; beim Agent-Aufruf kein anderes Modell und keine eingebauten Typen (`general-purpose`, `Explore`, `Plan`) waehlen. Haiku liefert Zuarbeit (Lesen, Entwurf, enge Teilaufgaben); Architekturentscheidungen, Gate-Auswertung und Abnahme macht die Hauptsitzung. Ist ein Subagent nicht noetig, die Arbeit direkt erledigen.

## Zuordnung
- Arbeit in genau einem Crate: `acta-<crate>-expert` (z. B. `acta-fs-expert`, `acta-xtask-expert`).
- Mehrere Crates: Querschnittsexperte
  - Loeschprozess, Messung, Berichte: `acta-loeschkette-expert`
  - Crash, Journal, Wiederanlauf: `acta-crash-recovery-expert`
  - Sperren, Threads, Async-Grenze: `acta-concurrency-expert`
  - Gates, rings.toml, Workspace-Regeln: `acta-governance-gates-expert`
  - Nachweise, Doku, Wortwahl: `acta-compliance-expert`
  - Latenz, Benchmarks: `acta-performance-expert`
  - Build, Toolchain, Lockfile: `acta-workspace-sanierung-expert`

## Auftrag formulieren
Der Subagent kennt nur seine Datei und das Repo, nicht dieses Gespraech. Der Auftrag nennt: Ziel, betroffene Pfade, SPEC-Kapitel und Zusagen, Abnahmekriterium (Befehle), was bereits ausgeschlossen ist. Mehrere unabhaengige Crates gleicher Ebene koennen parallel laufen; Crates verschiedener Ebenen nie (Ueberfuehrung laeuft L0 nach L9).

## Nach der Rueckmeldung
Ergebnis nicht ungeprueft uebernehmen: `/gates` selbst ausfuehren, Diff lesen, Aussagen des Experten ("Tests gruen") gegen den Lauf pruefen. Ein Experte darf keine Gates, Lints oder Budgets lockern.

## Pflege
Aenderungen an Expertenwissen zuerst in `docs/EXPERTEN-PROMPTS.md`, danach die Agent-Dateien daraus ableiten. Aussagen nur mit SPEC-Bezug; unbelegte Aussagen streichen.
