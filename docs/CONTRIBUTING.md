# Beitragen zu Contextra

<!-- doc-stamp: 2026-10-31 | Normativ: docs/SPEC.md Fassung 2.4, PLAN.md -->

Vielen Dank für Ihr Interesse an der Entwicklung von Contextra. Dieses Dokument beschreibt die verbindlichen Richtlinien, Qualitätsstandards und Entwicklungsabläufe für alle Beitragenden und Implementierer.

---

## 1. Voraussetzungen

- **Rust-Toolchain:** Version 1.89+ laut [`rust-toolchain.toml`](../rust-toolchain.toml) (Edition 2021).
- **Entwicklungsumgebung:** Optional `nix develop` mit [`flake.nix`](../flake.nix) für deterministische Systembibliotheken.
- **Workflow-Runner:** [`just`](../justfile) für vereinfachte Dev-Loops.
- **Normative Pflichtlektüre:** [`docs/SPEC.md`](SPEC.md) (Architektur, Invarianten) und [`PLAN.md`](../PLAN.md) (Roadmap, Stufen 0–3).

---

## 2. Entwicklungsablauf (TDD & Stufenplan)

1. **Arbeitspaket zuordnen:**
   Jede Änderung muss einem konkreten Arbeitspaket aus [`PLAN.md`](../PLAN.md) entsprechen:
   - **Stufe 0:** Bau wiederherstellen (Fehler B-01 bis B-12).
   - **Stufe 1:** Datenschutz- und Löschaussagen absichern (S-01 bis S-04, D-01 bis D-06, E0 bis E2).
   - **Stufe 2:** Härtung vor erstem Release (S-05 bis S-19, E3 bis E12).
   - **Stufe 3:** Scope-Bereinigung und Hygiene (Einfrieren nicht-benötigter Crates).
2. **Branch erstellen:**
   Schema: `wp/<nummer>-<kurzbeschreibung>`, z. B. `wp/0.1-segment-error-propagation`.
3. **Test-First (TDD):**
   Fehlerbehebungen und neue Funktionalität beginnen immer mit einem fehlschlagenden (roten) Unit- oder Integrationstest.
4. **Lokale Verifikation:**
   Vor jedem Push muss die Validierung lokal sauber durchlaufen:

```bash
# Mit just
just check
just test
just gates

# Oder mit reinem Cargo
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --locked --workspace
cargo test --test layering --locked
```

5. **Dokumentations-Synchronisation:**
   Ändern sich Signaturen, Invarianten, Crate-Klassen oder Ringe, MÜSSEN die relevanten Dokumente (`docs/SPEC.md`, `PLAN.md`, `capabilities.toml`, `docs/ARCHITECTURE.md`) im selben Commit bzw. PR aktualisiert werden.

---

## 3. Verbindliche Architektur- und Code-Regeln

- **Zero Panic (P7):** Absolutes Verbot von `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!`, `unreachable!` und ungeprüftem Array-Indexieren (`[i]`) im Produktionscode. Alle Fehler müssen als typisierte `Result<T, ContextraError>` zurückgegeben werden.
- **Unsafe-Beschränkung:** `unsafe` ist strikt auf die drei Inseln `contextra-sys`, `contextra-simd` und `contextra-wire` beschränkt. Jeder Block benötigt zwingend einen fundierten `// SAFETY:`-Beweiskommentar.
- **P28 Determinismus:** Keine direkten Aufrufe von `std::time::Instant::now()`, `SystemTime::now()` oder `thread_rng()` im Kerncode. Zeit, Zufall und IDs werden ausnahmslos über die Ports `Clock`, `Rng` und `IdGen` bereitgestellt.
- **Fail-Closed & Fehlerpropagation:** Dateisystem- und I/O-Fehler dürfen niemals ignoriert (`let _ =`) oder als Standardwert maskiert werden. Alle Fehler müssen via `?` propagiert werden.
- **Mandanten-Isolation (VETO-F10):** Jede Abfrage erzwingt die `TenantId` aus dem `RequestContext`. Literale, Standardmandanten oder mandantenübergreifende Caches sind unzulässig.
- **Keine Fake-Implementierungen:** Stubs, Mocks oder No-Op-Typen für Sicherheitsfunktionen (wie Null-KMS oder Null-PII-Filter) sind im Produktionscode verboten und gehören nach `contextra-testkit`.

---

## 4. Commit-Konventionen

Commit-Nachrichten sollten prägnant sein und bei Architekturänderungen die entsprechenden Trailer enthalten:

```gitcommit
feat(store): propagate deletion error in delete_segment

Behebt B-01: delete_segment gibt nun Result<bool, ContextraError>
zurück und schluckt keine Krypto-Revocation-Fehler mehr.

WP: 0.1
SPEC: §12.2.4
```
