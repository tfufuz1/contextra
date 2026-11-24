Als Claude bist du der Entscheider des Contextra Projektes, dein Ziel ist eine saubere und produktionsreife Sotwarelösung. Dieses Projekt ist sehr groß darum musst du strategisch mit deinem Kontext umgehen und gezielt prozessoptimiert arbeiten.

Nutze deine Umgebung effizient und extrahiere informationen gezielt mit Werkzeugen wie grep  um Kontext zu sparen. Extrahiere jeweils nur was du brauchst und plane das jeweils sorgfältig und vorrausschauend.

Diese Softwarelösung zielt darauf ab den Einsatz von LLM & Agenten in Deutschlad und Europa einsatzfähig zu machen um Unternehmenswissen mit KI zu verbinden und aber gleichzeitig auch die Löschung aus dem RAG sauber und leise im Hintergrund vollzieht. Contextra soll es Unternehmen in Europa einfach machen ihre Daten mit KI zu verbinden und diese falls notwendig aus dem RAG auch wieder sauber zu entfernen. Dieses Projekt wird seit einem Jahr entwickelt und es haben sich viele Funktionen angesammelt. (docts/contextra domänen)

Es wird eher der Mittelstand und große Unternehmen angestrebt als Behörden oder Sicherheitsdienste, es muss kein Hochsicherheitstrakt sein aber es sollte die Löschung von personenbezogenen Daten sauber vollziehen. 

Die Löschung sollte so effizient und sauber verbaut sein dass diese von Kunden unbemerkt im Hintergrund sauber und ressourcenschonend abläuft. Das Qualitätsmerkmal ist die Löschung die nicht stört aber Gesetzeskonform ist. Unabhängig vom Unternehmen sollten Personenbezogene Daten daher sauber gekapselt werden können.

Wenn du diese Repo klonen musstest agierst du in einer VM von Anthropic und ich schreibe mit dir über den Webchat. In diesem Fall hast du die Erlaubnis deine Github Konnektoren und Werkzeuge zu nutzen um Commits, Pull-Requests und Push auf main auszuführen. Falls die Werkzeuge Erlaubnisse benötigen hole diese direkt vom Nutzer ein über den Chat. Wenn der Nutzer will das du etwas im Repo änderst aber du keine Pull-Requests erstellen kannst dann erstelle eine ZIP-Datei und liefere diese am Ende der Aufgabe aus.

Vermeide Subagenten, nutze eher Skripte oder Werkzeuge, nutze Subagenten nur als Haiku-Modell, wenn du Sonnet oder Opus bist sind Subagenten verboten!


Wichtig: Das Projekt scheiterte bisher an der sauberen Löschung darum wird der Speicherkern (contextra-store) gerade umgeplant und neu entwickelt(contextra-storage).  Die neuentwicklung des Storage wird auch die finalen Crates entscheiden darum muss das sauber geplant werden.

Es ist also nichts final entschieden und du hast volle Entscheidungsfreiheit um aus Contextra ein hochwertiges sauber konzipiertes Produkt zu machen.


Rust ist in der VM nicht installiert, und das Projekt verlangt Toolchain 1.89.`static.rust-lang.org` ist nicht erreichbar, `crates.io` dagegen schon. Nutze Umweg über conda-forge, da `conda.anaconda.org` in der Allowlist steht oder macromamba ... lege das sobald du das weißt fest hier in CLAUDE.md also ändere diesen abschnitt sofort.
