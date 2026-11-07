# AGENTS.md — `contextra-adapt`

> **Ring 0** · Klasse `[K]` laut SPEC-Katalog (SPEC §0.5 archiviert es; Root-§10 K5: aktiv behandeln) · Reifegrad `beta` · Unsafe: nein · Rolle: Adaptive Algorithmen: PID-Latenzregler, Bandits, Drift-/Lyapunov-Wächter, Off-Policy.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-adapt --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `bandit-routing`, `flow-corrected-thompson`, `rie-greedy-personalization`, `sketched-bandit` |
| Lints | `#![forbid(unsafe_code)]`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Schulden

- Ist = Soll, autark.
- `pid_latency_controller.rs`, `homeostat.rs`, `decay_controller.rs` erscheinen auch in `db`/`engine` (Re-Exports/Kopien prüfen, DRY).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `pid.rs`, `pid_latency_controller.rs`, `homeostat.rs` | Regler |
| `bandit.rs`, `flow_thompson.rs`, `rie_greedy.rs`, `off_policy.rs` **und** `offpolicy.rs` | Bandit-/Policy-Lernen (zwei Off-Policy-Dateien: Dopplung klären) |
| `drift.rs`, `lyapunov.rs`, `shadow_mode.rs`, `decay_controller.rs` | Drift, Stabilität, Schattenmodus |

## 4. Verträge & Invarianten

1. **Determinismus (P28):** Zufall nur über `Rng`, Zeit nur über `Clock` (Heuristik: 1 `Instant::now`/`SystemTime::now`-Treffer prüfen).
2. **Wirkungsgrenze:** Regler liefern Empfehlungen/Parameter; sie schreiben nie selbst in Speicher.
3. Alle Regler haben Grenzen (Clamp), Anti-Windup und einen Fallback auf feste Startwerte (SPEC §4.7).

## 5. Ports

Implementiert `AdaptiveController`-artige Rollen; konsumiert `Clock`, `Rng`.

## 6. Nebenläufigkeit & Performance

- Zustand je Regler in der Instanz, kein globaler Zustand; Aufruf aus einem Thread oder hinter Nachrichtenkanal.

## 7. Anti-Patterns

- Globale Singleton-Regler.
- Stille Anpassung der Ranking-Gewichte ohne Schattenmodus und Messung.

## 8. Tests & Messung

```bash
cargo test -p contextra-adapt --locked
cargo clippy -p contextra-adapt --all-targets -- -D warnings
```

- Konvergenz-/Stabilitäts-Property-Tests, deterministischer Replay mit `SeededRng`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | `graph → adapt`-Kante ist in `capabilities.toml` noch deklariert, in Cargo nicht mehr: bereinigen |
| DRY | `offpolicy.rs` vs `off_policy.rs` zusammenführen |
