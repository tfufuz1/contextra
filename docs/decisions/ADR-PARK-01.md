# ADR-PARK-01: Entfernen statt Einfrieren von Parkplatz-Funktionen

Status: Vorgeschlagen (wird mit Merge der Welle 1 akzeptiert)
Datum: 15.10.2026
Bezug: Gesamtspezifikation v4.1, Abschnitte 4.5, 10.1, 11, 12, 14, I.3

## Kontext
Der Hauptzweig enthält Funktionen der Endprodukte EP2 bis EP6 sowie Querschnittsfeatures (SSN, ACI, DiBud, KIVI, SnapKV, LinUCB, Spektral-SVD, py, Optimierungen), deren Umfang die Stabilitätskriterien S1 bis S12 gefährdet. Die Prüfung der Agenten leidet unter der Breite der Fläche (Halluzinationsrisiko), und eingefrorener Code bleibt im Build, im Gate-Lauf und im Abhängigkeitsbaum.

## Entscheidung
1. Features, die den Kern destabilisieren, werden aus dem Hauptzweig entfernt, nicht durch Flags oder cfg gehalten.
2. Vor jeder Entfernung wird ein annotierter Tag `park/<ID>/v1` auf dem letzten Stand gesetzt, der das Feature enthält.
3. Die Entfernung ist ein eigenes Arbeitspaket mit eigener Karte und eigenem PR.
4. Eine Wiedereinführung erfolgt nur über einen eigenen PR, der den Tag als Quelle nennt und die Voraussetzungen aus 11.1 sowie S1 bis S12 nachweist.
5. Im Hauptzweig existiert nur Code mit Produktionsaufrufer und Integrationstest über den echten Pfad (PE5).

## Umfang der Entfernung in Welle 1
- Tags: park/EP2/v1 bis park/EP6/v1, park/SCOPE-EXTRA/v1 (SSN, ACI, DiBud, KIVI, SnapKV, LinUCB, Spektral-SVD, py, Optimierungen)
- Mitgliedschaft in der Workspace-Liste entfernen (Schritt 3a), Dateibaum bleibt bis zur Messung
- Erhalten bleiben: Hybridsuche, Mandanten, Löschung Off und Assured, MCP-Kern, Fassade EP1

## Lockerung der Drei-PR-Grenze
Für Welle 1 gilt abweichend von Abschnitt 12: bis zu sieben parallele PRs, sofern jeder genau ein Crate oder eine Gate-Datei berührt und alle Scopes disjunkt sind. Die Reihenfolge der Merges ist: J5 und J6 zuerst, dann J1, dann J2, danach J3 und J4.

## Konsequenzen
- Die Hülle wird ohne Feature-Flags gemessen, nicht durch Abschaltung.
- Wiedereinführung ist möglich, aber nur mit Nachweis.
- Git-Historie und Tags sind die Quelle der Wahrheit für entfernten Code.

## Verworfene Alternativen
- Einfrieren mit Feature-Flags: bleibt im Build und Abhängigkeitsbaum, erhöht Halluzinationsfläche.
- Eigenes Repository je Parkplatz: Pflegeaufwand ohne Nutzen vor K4.
