# AGENTS.md — `contextra-testkit`

> **Werkzeug** · Klasse Werkzeug · Reifegrad `stable` · Unsafe: nein · Rolle: Test-Doubles und Konformitätssuiten: `FaultVfs`, `ManualClock`, In-Memory-Stores, Referenzmodell.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-testkit --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `testkit` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Nur als `dev-dependency`**; nie in Release-/Default-Features, nie Abhängigkeit eines Kerns im Normalbau.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `fault_vfs.rs`, `manual_clock.rs` | Fehlerinjektion, deterministische Uhr |
| `in_memory_{store,vector,text,graph}.rs` | Referenz-Backends |
| `reference_model.rs` | deterministisches Referenzmodell für MVCC/KV |
| `conformance/` (`clock`, `rng`, `storage`, `vector`, `text`, `graph`, `metrics`, `vfs`, `failure`) | Konformitätssuiten für Fremdadapter |
| `examples/standalone_testkit.rs` | Autarkes Baukasten-Minimalbeispiel (M10) |
| `mocks.rs` | Mocks (nie Produktionsstandard, INV-EMBED-1) |

## 4. Verträge & Invarianten

1. Doubles verhalten sich **wie die Spezifikation**, nicht wie die Implementierung (Orakel unabhängig vom Erzeuger, INV-ORACLE-1).
2. Deterministisch: keine Systemzeit, kein `thread_rng`.
3. Konformitätsfunktionen sind für Fremdprojekte nutzbar (Baukasten-Beweis).

## 5. Ports

Implementiert Test-Varianten von `Vfs`, `Clock`, `StorageEngine` u. a.

## 6. Nebenläufigkeit & Performance

- Keine Hintergrund-Threads ohne Join.

## 7. Anti-Patterns

- Mock als Produktionsimplementierung.
- Systemuhr in Tests.

## 8. Tests & Messung

```bash
cargo test -p contextra-testkit --locked
cargo clippy -p contextra-testkit --all-targets -- -D warnings
```

- Eigene Tests für die Doubles; Mutation-Score anstreben.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.5 | Konformitätssuiten für alle Ports |
