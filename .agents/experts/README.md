# Fachprofile für Agenten

Nicht normativ. Normativ ist ausschließlich [`docs/SPEC.md`](../../docs/SPEC.md) (Version 4.3.0); bei Widerspruch gilt die SPEC (D-98). Diese Dateien leiten Arbeitsweise ab und enthalten keine eigenen Anforderungen.

## Aufbau der Kontextkette

| Ebene | Datei | Inhalt |
|---|---|---|
| 1 | `AGENTS.md` (Wurzel) | gemeinsame Regeln, Ringe, Ladeordnung |
| 2 | `crates/<crate>/AGENTS.md` | Fachprofil, normative Anker, Verbote, Abnahme **dieses** Crates |
| 3 | `.agents/experts/querschnitt.md` | Profile für Arbeit über mehrere Crates (Lösch-Kette, Crash, Concurrency, Build, Compliance, Performance) |
| 4 | `.agents/rules/*.md` | knappe Regeln je Themengebiet |
| 5 | `.agents/jules/` | Ablauf, Arbeitsliste, Stopp-Katalog (nur Jules-Sitzungen) |

Ein Agent arbeitet im Fachprofil des Crates, in dem er ändert. Das Profil ersetzt Einzelanweisungen: Wer als Durability-Ingenieur arbeitet, wiederholt nach einem `fsync`-Fehler nie auf demselben Handle, weil das zum Fachwissen gehört (und D-55 es verlangt).

## Gemeinsamer Kopf (jedem Profil vorangestellt)

```
Normativ ist docs/SPEC.md (4.3.0); AGENTS.md und dieser Prompt sind es nicht. Bei Widerspruch gilt die SPEC.
Du arbeitest an Contextra: eingebettete Rust-Bibliothek (Single-Node, kein Server) für lokales Wissen und Gedächtnis
von Agenten- und RAG-Systemen mit gemessenem, unabhängig prüfbarem Löschprozess.

Harte Regeln: kein Netzwerk, keine Telemetrie, kein Key-Escrow; Verschlüsselung und WAL-Integrität nicht abschaltbar;
kein unwrap/expect/panic!/unreachable!/`let _ =` auf Result im Produktionscode; unsafe nur in contextra-sys und
contextra-simd; kein mmap; I/O nur über den Port Vfs; Zeit, Zufall und IDs nur über Clock, Rng und IdGen; fsync-Fehler
nie auf demselben Handle wiederholen; Abhängigkeiten nur gemäß Matrix SPEC 5.3; keine Agenten- oder Sitzungsmarken im Code.
Löschaussagen: ausschließlich der Wortlaut aus SPEC 6.4.

Arbeitsweise: erst fehlschlagender Test mit SPEC-ID, dann Implementierung. Jedes Symbol vor Verwendung verifizieren,
Prüfbefehle tatsächlich ausführen, Ergebnisse ehrlich berichten (nicht Gelaufenes als „nicht geprüft“ kennzeichnen).
Offene Annahmen benennen statt stillschweigend entscheiden. Keine Gates, Lints oder Baselines lockern, um Rot zu beseitigen.
```

## Bewusste Abgrenzung zur Löschaussage

Formulierungen wie „Hardware-Kill-Switch“, „in Millisekunden, unabhängig von der Systemlast“ oder „mathematisch belegbare Datensouveränität“ sind **nicht** Teil der Profile. Sie widersprechen SPEC 6.4 (einzige zulässige Außenaussage; Flash-Remanenz, Dateisystem-Journale und Prozessspeicher sind ausdrücklich nicht erfasst), 1.6 und D-30. Sie würden zu unzulässigen Löschaussagen in Code, Doku und Fehlermeldungen verleiten (T-LINT-1).

Richtig ist: O(1) bezüglich der Zielgröße gilt für den Pfad bis `Gated`, also für die Schlüsselvernichtung (P16, D-79, BM-ERASE-O1), nicht für das gesamte Löschen. Der Zielwert ist plattformrelativ: p50 ≤ 10 ms auf der NVMe-Referenzplattform, allgemein ≤ 6 × `fsync_p50` + 2 ms (D-30).
