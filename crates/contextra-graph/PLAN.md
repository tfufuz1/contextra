# contextra-graph — Umsetzungsplan (Entkopplung & Baustein-Architektur)

> Abgeleitet aus `docs/SPEC.md` v2 §7 und dem Baukasten-Profil für Klasse A Bausteine.
> Ziel: Transformation in einen autark einbettbaren, hochperformanten Graph-Baustein.

---

## 1. Übersicht & Zielarchitektur

`contextra-graph` wird von harter Verdrahtung befreit und als eigenständiger Graph-Baustein etabliert:
- **Phase 1: Spezifikation & Vertragsfestlegung:** Definition neutraler Interfaces (`contract.rs`), Fehlertypen (`error.rs`) und lokaler Domänentypen (`types.rs`).
- **Phase 2: Kompatibilitäts-Kapselung:** Verlagerung aller `contextra-ports` und `contextra-types` Abhängigkeiten in `src/compat/` hinter den Cargo-Features `types-compat` und `ports-compat`.
- **Phase 3: Umfassendes Kommentierungs- und Anchor-System:** Vollständige Durchdringung des Quellcodes mit standardisierten `FILE-CONTEXT` Headern, hierarchischen `SECTION` / `!SECTION` Regionen und eindeutigen `ANCHOR[contextra-graph:<TAG>]` Markern nach dem Comment-Anchors Schema.
- **Phase 4: Qualitätssicherung & Modulare Verifikation:** Sicherstellung, dass Unit-Tests, Integrationstests und Benches fehlerfrei laufen und der Decoupling Inspector einen klaren Pfad zur Entkopplung ausweist.

---

## 2. Detaillierte Arbeitspakete

| Arbeitspaket | Inhalt | Verifikation | Status |
|---|---|---|---|
| **WP-GRAPH-1** | **Spezifikation & Modulgrenzen**<br>- Aktualisierung von `SPEC.md` und `PLAN.md`<br>- Definition von Ring 1 / Klasse A Prinzipien und Invarianten | `SPEC.md` & `PLAN.md` synchronisiert | In Bearbeitung |
| **WP-GRAPH-2** | **Tooling für Comment-Anchors**<br>- Entwicklung von `comment_anchors.py` zur Validierung und Injektion | Script lauffähig in `/home/jules/self_created_tools/` | Geplant |
| **WP-GRAPH-3** | **FILE-CONTEXT & Sektionierung**<br>- Header (`FILE-CONTEXT`, `STAND`, `ZWECK`, `INVARIANTEN`) für alle 83 Source-Dateien<br>- Strukturierung in zusammenklappbare `SECTION` Blöcke | Audit-Prüfung aller Dateien | Geplant |
| **WP-GRAPH-4** | **Inline-Dokumentation & Anchors**<br>- Detaillierte Erläuterungen komplexer Algorithmen (CSR, PPR, Louvain/Leiden, Session DAG)<br>- Vergabe kollisionsfreier `ANCHOR[contextra-graph:<TAG>]` IDs | Zero-Panic Check, Anchor-Eindeutigkeit | Geplant |
| **WP-GRAPH-5** | **Architektur-Guide & Readme**<br>- Ausführliches `README.md` und Moduldokumentation in `src/lib.rs`<br>- Schnelleinstieg und Integrationsbeispiele für Entwickler | Doctest & Markdown-Prüfung | Geplant |
| **WP-GRAPH-6** | **Kompilierung, Tests & Gates**<br>- `cargo check -p contextra-graph`<br>- `cargo test -p contextra-graph`<br>- Prüfung auf Warnungsfreiheit und Lints | CI/Build grün | Geplant |

---

## 3. Definition of Done

1. Alle 83 Quellcode-Dateien unter `crates/contextra-graph/src/` verfügen über den standardisierten `FILE-CONTEXT`-Kopf und `SECTION`-Regionen.
2. Kern-Algorithmen (CSR Traversierung, Bitemporale Sichtbarkeit, Forward-Push PPR, Community Detection, Session DAG) sind transparent und zeilengenau mit `NOTE:`, `TODO:`, `FIXME:` und `ANCHOR` dokumentiert.
3. Die Entkopplungsprinzipien sind in `README.md`, `SPEC.md` und `PLAN.md` konsistent beschrieben.
4. Alle Tests in `crates/contextra-graph` kompilieren und bestehen fehlerfrei.
