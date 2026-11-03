# AGENTS – contextra-testmacros

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (15.1, T-TRACE-1). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Proc-Macros für Contextra-Tests und Benchmarks (`ctx_test`, `ctx_bench`).

**Ring und Abhängigkeiten:** Ring dev (Werkzeug, `proc-macro = true`). Nur als `[dev-dependencies]` einzubinden; kein Produktionscrate hängt daran. Keine Contextra-Crates als Abhängigkeit.

**Hinweis zur SPEC:** Dieses Crate ist in SPEC 5.1 und 5.1a nicht aufgeführt, die Rückverfolgbarkeit (T-TRACE-1) verlangt aber das Attribut `#[ctx_test(id = "T-…")]`. Eine Ergänzung der SPEC-Tabellen (zuerst SPEC, dann Folgedokumente) ist offen; bis dahin ist der Zweck-Satz der aus `Cargo.toml`.

## Fachprofil (Arbeitsrolle)

Proc-Macro-Ingenieur für Test-Infrastruktur. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Fehlermeldungen zur Compile-Zeit mit `compile_error!` und präziser Span, `trybuild`-Tests für Pass und Fail.
- Hygiene: keine Seiteneffekte auf Laufzeit und keine versteckten Abhängigkeiten im erzeugten Code.

## Normative Anker

- `#[ctx_test(id = "T-…")]` und `#[ctx_bench(id = "BM-…")]` validieren die ID-Form zur Compile-Zeit. `tests/arch` liest `docs/SPEC.md`, extrahiert alle `T-`/`BM-`/`I-`/`FM-`-IDs und prüft in beide Richtungen, dass jede ID einen Test hat und jeder Test eine existierende ID nennt (T-TRACE-1).
- Ausnahmen nur mit Ablaufdatum in `governance/exceptions.toml`.

## Verboten

Einbindung in Produktionscrates; IDs ohne SPEC-Bezug; Abschwächen der ID-Prüfung, um Rot zu beseitigen.

## Abnahme

`trybuild`-Fälle `pass` und `fail_*` (Verzeichnis `tests/ui/`): fehlende ID und ungültiges Präfix werden abgelehnt; `T-TRACE-1` in `tests/arch`.

## Prüfbefehle

```bash
cargo test -p contextra-testmacros --locked
cargo test -p contextra-arch-tests --locked
```
