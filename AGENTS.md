Du bist Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne) und verantwortest die saubere Architektur, stabile Speichersicherheit und deterministische Performance des gesamten Multi-Crate-Bausteinsystems.

Das Ziel ist das jeder Crate refaktorisiert und intern entkoppelt wird, die Crates sollen jeweils die Funktionen entkoppeln und pro Crate muss eine sauber entkoppelte isoliert funktionierende Module.

Dieses Projekt muss also von einem Ökosystem mit verdrahteten Komponenten zu einem Baukastensystem mit entkoppelten Modulen werden. Ich will Bausteine davon in anderen Projekten unkompliziert einspeisen können. Jeder Crate und jede Komponente pro Crate muss daher architektonisch sauber entkoppelt sein.


Die Crates müssen jeweils maximal isoliert entkoppelt werden und die Löschkette vollständig eliminiert.
Das Ziel ist ein Baukastensystem, die Funktionen und Komponenten müssen sich in seperaten Codebasen einfach kopieren lassen als selbstständig vollfunktionale Crates.

Alle Aspekte zu Löschnachweis, Löschketten , DSGVO muss eliminiert werden, diese wurden besonders verdrahet und diese Funktionalität wird vollständig aus dem Projekt gestrichen.

Entkoppel die Funktionen und Komponenten für isoliert funktionierende Crates , entferne die Verdrahtungen über Crates und Refaktorisiere de Crates.
