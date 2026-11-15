---
name: acta-delivery
description: "Erzeugt und liefert die ZIP-Datei mit allen bearbeiteten Dateien (Pfade ab Repo-Wurzel, entfernte Dateien als leere Huellen). Verwenden nach jedem abgeschlossenen Arbeitspaket, auf Anforderung (/deliver) und zwingend bevor Kontext oder Kontingent enden."
---

# Lieferung als ZIP

Der Nutzer ueberschreibt lokal mit dem ZIP-Inhalt. Deshalb gilt:
- Nur **bearbeitete oder neue** Dateien, Pfade ab Repo-Wurzel (z. B. `crates/acta-fs/CLAUDE.md`).
- **Entfernte** Dateien als **leere Dateien** mit ihrem alten Pfad; der Nutzer loescht diese Huellen lokal. Archivordner, die nach erfolgreicher Ueberfuehrung entfernt werden, werden so geliefert.
- Nichts anderes: kein `target/`, kein `.git/`, keine Logs, keine unveraenderten Dateien.

## Ablauf
1. `/gates` ausfuehren (Skill `acta-gates`); Stand ehrlich festhalten.
2. `bash scripts/make-delivery.sh` (Ziel `/root/acta-lieferung.zip`, ausserhalb des Repos). Das Skript nimmt `git ls-files -m -o --exclude-standard` und `git ls-files -d`; bei Aenderungen an ignorierten Dateien (`.gitignore`) diese Dateien vorher mit `git add -N` sichtbar machen oder gezielt pruefen.
3. Inhalt kontrollieren: `unzip -l /root/acta-lieferung.zip` (Anzahl, keine Fremddateien, geleerte Dateien mit Groesse 0 nur dort, wo wirklich Entfernung gewollt ist).
4. Mit `SendUserFile` senden (`status: normal`, wenn der Nutzer gerade im Gespraech ist, sonst `proactive`) und kurz nennen: Anzahl, Pruefstand, Offenes.

## Zeitpunkt
- Nach jedem Arbeitspaket (Zwischenlieferung).
- **Vor Ablauf des Kontingents:** sobald Kontext oder Kontingent knapp werden oder eine lange Aufgabe noch aussteht, sofort liefern und danach weiterarbeiten. Nicht bis zum Schluss warten.
- Bei Abbruch der Arbeit: den letzten gruenen Stand liefern, offene Punkte benennen.

## Hinweise
- Ein ZIP ersetzt keine Pruefung: Nicht Gelaufenes als "nicht geprueft" kennzeichnen.
- Sub-Agenten nur mit `model: haiku` (siehe CLAUDE.md, Arbeitsregeln).
