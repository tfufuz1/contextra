# AGENTS – tests/arch

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (5.2, 5.3, 5.8, 15.1 „Architektur und Lint“, 15.3, D-90, D-91, D-97, D-98). Bei Widerspruch gilt die SPEC. Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Architektur-, Ring-, Namens- und Lint-Tests.

**Paket und Ring:** Paket `contextra-arch-tests`, Ring dev. Wird von keinem Produktionscrate referenziert. Der Zweck-Satz laut SPEC 5.1a muss wortgleich in `Cargo.toml` (`description`), im `//!`-Kopf und in der ersten Zeile der README stehen (T-ARCH-5); die vorhandene `description` weicht ab und ist in WP-1.5 anzugleichen.

## Fachprofil (Arbeitsrolle)

Architektur-Governance-Ingenieur (statische Analyse, `cargo metadata`, `syn`, Ratchet-Verfahren). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Ringmatrix-Prüfung, fail-closed bei unbekannten Crates; Verbote für `unsafe`, `std::fs` und Netzwerk; Manifest-, Header- und Namenskonventionen; Wortlaut-Lint; Feature-Hygiene.
- Ratchet für bekannte Abweichungen (`known_deviations`): nur abbauen, nie erweitern.

## Normative Anker (Testfamilien, 15.1)

| ID | Prüfgegenstand |
|---|---|
| T-ARCH-1, T-ARCH-2 | Ring-DAG und Matrix (5.2, 5.3); `types` ohne `async` und I/O (AR-5); nur `TenantScopedStorage` ruft den rohen Store (AR-6) |
| T-ARCH-3 | `erasure` ohne Ring-2-Kanten (AR-3); `lab` und `ctx-verify` ohne Produktions-Decoder (AR-4); `store` ohne `text`/`vector`/`erasure` (AR-2); entfernte Crates tauchen nicht auf (D-24) |
| T-ARCH-4 | Kein Netzwerk-Crate im Produktionsbaum (`cargo tree -e normal`, D-54, HR-1) |
| T-ARCH-5 | Ringangabe, Abhängigkeiten und Zweck-Satz im Crate-Kopf gegen 5.1, 5.1a, 5.3 (D-91, D-97) |
| T-ARCH-6 | Matrix aus `cargo metadata`; kein Glob-Re-Export in `ports` (AR-10); Dev-Abhängigkeiten nach AR-14 |
| T-ARCH-7 | Fassade ohne Logik (AR-12); Namens- und Modulregeln (D-90); keine Agenten- oder Zeitstempelmarken (T-LINT-4) |
| T-ARCH-8 | Keine Sitzungsverläufe, Prüfprotokolle, `reports/`, `evidence/` im Quellbaum (D-98 (3)) |
| T-UNSAFE-1 | `unsafe` nur in `sys` und `simd`, sonst `forbid(unsafe_code)`; keine safe-Funktion mit offener Vorbedingung |
| T-CFG-1 bis T-CFG-3 | jedes `feature = "…"` ist deklariert, kein `allow(unexpected_cfgs)`, kein Sicherheitsschalter (AR-9, HR-13) |
| T-LINT-1 bis T-LINT-4 | Wortlaut-Lint (SPEC.md ausgenommen), kein `block_on`, kein zweites `Vfs`, alte Begriffe entfernt, keine Marken |
| T-TRACE-1 | Rückverfolgbarkeit Spezifikation ↔ Test |
| T-DOC-1 | Abgeleitete Dokumente: wortgleiche Zweck-Sätze, keine Begriffe aus 6.4 außerhalb der SPEC, existierende Pfad-, Workflow- und Crate-Verweise, keine Zeitstempel, Agentenkennungen, absolute Pfade |
| T-SUPPLY-1, T-SUPPLY-2 | Lieferkette, SBOM, Toolchain-Regel (D-82) |

- Ratchet-Zähler für `unwrap`/`expect`/`panic!`/`let _ =`/`#[allow]` sind monoton fallend (5.8). Basiswerte: `.agents/governance/ratchet.toml`; befristete Ausnahmen: `governance/exceptions.toml` mit Ablaufdatum und SPEC-Bezug.
- Neue Regel oder neue Ausnahme: zuerst SPEC-Bezug nennen. Regeln werden nie geschwächt, um Rot zu beseitigen.

## Verboten

Schwächen oder Löschen einer Regel, Erweitern von `known_deviations`, Ausnahmen ohne Ablaufdatum, Schreibzugriff auf `docs/SPEC.md`, Abhängigkeit von Produktionscode außer über `cargo metadata` und Quelltext-Analyse.

## Abnahme

Die Prüfung wird rot bei einem absichtlich eingebauten Verstoß (Selbsttest je Regel). Jede Regel nennt in ihrer Fehlermeldung die SPEC-ID.

## Arbeitspakete

WP-0.1 (Grundgerüst, Wortlaut-Lint, T-TRACE-1), WP-1.5 (T-ARCH-5 bis T-ARCH-8, T-DOC-1), WP-0.2 (Lieferkette). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-arch-tests --locked
cargo test -p contextra-arch-tests --locked --test arch
```
