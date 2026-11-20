- **Rust-Installation:** `crates.io` erreichbar, `static.rust-lang.org` nicht. Nutze `conda-forge` (`conda.anaconda.org` erlaubt) mit micromamba oder `apt` mit `rustc-1.89` / `cargo-1.89`.

Wenn du diese Repository klonen musstest bist du in einer VM von Anthropic ohne Github Zugangsdaten, dann musst du deine Ergebnisse wenn du fertig bist oder das Kontingent 90% erreicht hat als ZIP-Datei mit Ordnerstruktur im Chat zum Download ausliefern. Falls Dateien entfernt werden liefere diese als leere Dateien mit dazu damit ich das überschreiben kann. Du hast die vollste Erlaubnis deine Änderungen direkt in Github zu pushen, wenn das möglich ist. Über Github direkt wäre besser.


Die Spezifikation in docs/SPEC.md ist die einzige Quelle der Wahrheit für den Soll-Zustand des Projekts. Die PLAN.md pro Crate müssen noch darauf angepasst und jeweils pro Crate spezifiziert werden als Implementierungsspezifikation mit exakten Änderungen wie Entfernungen, Refaktorisierungen und Implementierungen.




Die Test- und Bench-Targets sind in vielen Crates kaputt: contextra-engine (≈ 60 Tests), contextra-vector (≈ 25 Tests), contextra-checkpoint, contextra-erasure, contextra-text.