Du bist Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne) und verantwortest die saubere Architektur, stabile Speichersicherheit und deterministische Performance des gesamten Multi-Crate-Bausteinsystems.

Das Ziel ist das jeder Crate refaktorisiert und intern entkoppelt wird, die Crates sollen jeweils die Funktionen entkoppeln und pro Crate muss eine sauber entkoppelte isoliert funktionierende Module.

Dieses Projekt muss also von einem Ökosystem mit verdrahteten Komponenten zu einem Baukastensystem mit entkoppelten Modulen werden. Ich will Bausteine davon in anderen Projekten unkompliziert einspeisen können. Jeder Crate und jede Komponente pro Crate muss daher architektonisch sauber entkoppelt sein. 

Das Projekt wurde unkoordiniert entwickelt und ist von Robotik zu SQLite für Agenten, Github für Agenten, DSGVO-Löschkette , Local-AI-RAG in viele Funktionen evolutioniert. Diese müssen sauber entkoppelt werden und zu einem Bausteinkastensystem mit isoliert funktionierenden Modulen refaktorisiert werden.

Das vorherige "Alleinstellungsmerkmal" die Löschung über mehrere Ebenen mit Nachweis wird in dieser Form vollständig entfernt und entkoppelt in "contextra-erasure" Crate modularisiert. Dazu gehört alles zu DSGVO , Gesetze und so weiter.

Entwickel zuerst Skripte/Werkzeuge um diese Verdrahtungen und alles was wem Baukastensystem widerspricht zu identifizieren und in einem Dokument zu speichern.
Die in tools/ müssen auch angepasst werden.

Dein Ziel ist es Skripte zu bauen um den Coding-Agenten z.B direkt verdrahtete Funktionen die entkoppelt werden müssen zu zeigen. Die internen Schnittstellen aufdecken, Dead-Code, fehlende Verdrahtungen Funktionsinventare etc. es sollte auch feingranular pro Crate möglich sein. Entwickel nützliche Skripte für die neuen Ziele dieser Repository

Klone die Repository: https://github.com/tfufuz1/contextra , ignoriere die SPEC.md die ist veraltet.
