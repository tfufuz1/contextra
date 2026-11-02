# Grundsätze und Governance

Abgeleitet aus [`SPEC.md`](SPEC.md) Kapitel 2 (Leitprinzipien), 6.3 (harte Regeln) und 6.6 (Rechtsentscheidungen). Bei Widerspruch gilt die SPEC. Dieses Dokument enthält keine eigenen Regeln.

## 1. Rangfolge

1. `docs/SPEC.md` ist die einzige normative Quelle (SPEC-Kopf, Geltung).
2. `docs/VETOES.md` konkretisiert verbindliche Verbote.
3. `docs/CONSTITUTION.md`, `docs/ARCHITECTURE.md`, `docs/TYPE_REGISTRY.md`, `docs/COMPILATION.md`, `docs/CONTRIBUTING.md`, `docs/SECURITY.md`, `AGENTS.md`, `README.md` sind abgeleitet. Sie werden bei Widerspruch korrigiert.
4. Der Quellcode folgt der SPEC. Abweichungen sind Defekte.

## 2. Leitprinzipien (SPEC 2)

| ID | Prinzip |
|---|---|
| P1 | Wirkung vor Behauptung: Ein Status „gelöscht“ entsteht nur aus einem Zeugen mit Positivkontrolle. |
| P2 | Fail-closed: Unbekannte Zustände führen zu Verweigerung oder `Pending`. |
| P3 | Ein Weg: ein Löschpfad, ein Format, eine Fehlerbehandlung je Konzept. Keine Legacy-Pfade. |
| P4 | Ziel-Isolation: Nachbardaten bleiben auf Record-Ebene bitgleich. |
| P5 | Keine Schalter für Sicherheit. Ausnahmen: `process_hardening` und das Feature `testing`. |
| P6 | Keine Panics, keine verschluckten Fehler. |
| P7 | Determinismus über Ports: Zeit, Zufall, Dateisystem, IDs. Ausnahme: Schlüssel-, Salt- und Nonce-Material aus dem OS-CSPRNG. |
| P8 | Kleinster Kern: Ein Crate oder Modul gehört nur in den Kern, wenn ein Kernkonsument es braucht. |
| P9 | Messen vor Optimieren: Jedes Leistungsgate hat eine Basislinie auf dokumentierter Hardware. |
| P10 | Verifizierbarkeit vor Eleganz: Jede Entscheidung nennt den belegenden Test. |
| P11 | Löschbarkeit als Strukturprinzip: Physischer Aufwand skaliert mit dem Ziel. |
| P12 | Entkopplung: Der Löschkern kennt nur Traits aus `ports`. |
| P13 | Datensparsamkeit im Nachweis: Nur Commitments, Bereichs-Tags, Zähler und Zeiten. |
| P14 | Dauerhaftigkeit ist ein Vertrag mit dem Betriebssystem. |
| P15 | Betreibbarkeit ist Teil des Produkts: Gesundheitsstatus, Fehlertexte, Runbooks. |
| P16 | Sofortwirkung ist zielgrößenunabhängig. |
| P17 | Ehrliche Schicht-Reichweite: Keine Aussage über „das System“ aus der Wirkung auf einen Teil. |

## 3. Harte Regeln (SPEC 6.3)

| ID | Regel |
|---|---|
| HR-1 | Kein Netzwerkzugriff im Produktionsbuild. |
| HR-2 | Keine Telemetrie, kein Crash-Upload, kein Phone-Home. |
| HR-3 | Kein Schlüssel-Escrow durch den Hersteller. |
| HR-4 | Fehler- und Lognachrichten enthalten keine Inhalte, Kennungen, Schlüssel oder personenbezogenen Pfade. |
| HR-5 | Außenaussagen ausschließlich nach SPEC 6.4. |
| HR-6 | Jede Zeile der Leistungsbeschreibung hat einen automatisierten Test und einen Evidenzlauf. |
| HR-7 | Ein Erase, das Daten außerhalb des Ziels verändert, ist Release-Blocker ohne Ausnahme. |
| HR-8 | Sicherheitslücken werden innerhalb dokumentierter Fristen bearbeitet. |
| HR-9 | Haftungsausschlüsse der Lizenz gelten nicht als Schutz. |
| HR-10 | Löschzusagen enthalten nie einen Zeugen ohne Positivkontrolle. |
| HR-11 | Kein Release ohne bestandenes Gate G-H und signierte Artefakte samt SBOM. |
| HR-12 | Kein neuer `unsafe`-Block, keine neue Abhängigkeit, kein neues Feature ohne belegenden Test. |
| HR-13 | Kein Schalter schwächt Verschlüsselung, Integrität, Löschmessung oder Mandantenprüfung. |

## 4. Rechtlicher Rahmen

- Verantwortlicher im Sinne der DSGVO ist der Betreiber. Contextra ist bei bestimmungsgemäßem Betrieb weder Verantwortlicher noch Auftragsverarbeiter (SPEC 6.1, E-L6).
- Contextra macht keine Rechtsaussage über die Wirkung einer Löschung. Die Wertung der Schlüsselvernichtung bleibt beim Betreiber (E-L2).
- Die Verbotsliste für Außenaussagen steht in SPEC 6.4. Die Einhaltung prüft der Wortlaut-Lint (T-LINT-1).
- Rechtsfragen, die offen sind, stehen in SPEC 6.7 (G-LEGAL). Dieses Gate blockiert nur Löschaussagen über 6.4 hinaus.

## 5. Governance

- **Änderungen der SPEC** erfolgen mit einem Eintrag in Anhang H und mit Bestätigung durch den Eigentümer der Spezifikation (Anhang H, D-93 bis D-98).
- **Arbeitspakete und Gates** sind in SPEC 15.3 und 15.4 festgelegt. Ein Phasenwechsel ohne bestandenes Gate ist unzulässig.
- **Ratchet.** Bestehende Verstöße gegen `unwrap`, `expect`, `panic!` und `let _ =` dürfen nur sinken (SPEC 5.8). Die Datei `governance/ratchet.toml` wird mit WP-0.1 angelegt.
- **Keine Schattenwahrheiten.** Neben der SPEC gibt es keine weiteren normativen Pläne, Vorfassungen oder Agentenregeln. Abgeleitete Dokumente werden gepflegt, nicht gelöscht; ihr Inhalt muss der SPEC entsprechen (D-98).

## 6. Qualitätsgrundsätze

- Rot vor Grün: Ohne fehlschlagenden Test kein Merge (D-44).
- Ein Löschpfad, eine Fehlerbehandlung je Konzept (P3).
- Einheitliche Namen: keine Module namens `misc`, `util`, `utils`, `common`, `helpers` oder mit der Endung `_impl` (D-90).
- Ein Modul, eine Form: `foo.rs` neben `foo/`, kein `mod.rs` (D-90).
