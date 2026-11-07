# AGENTS.md — `contextra-privacy`

> **Ring 3** · Klasse `[R]` (Rand) · Reifegrad `stable` · Unsafe: nein · Rolle: PII-Erkennung, Egress-Gateway/-Guard, Exfiltrationserkennung, Verarbeitungsverzeichnis.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-privacy --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll; `capabilities.toml` nennt zusätzlich `avv-generator` (nicht in Cargo).
- 6 Treffer mit Shredding-Bezug: prüfen (Zuordnung `erasure`/`kvcache`).
- Soll: implementiert den Port `EgressPolicy` (WP 2.4), der noch **nicht** in `ports` existiert (hier nur `BlockReason::EgressPolicyDenied`).
- `mcp` re-exportiert `egress_*`/`bulk_exfiltration_detector` (dünne Wrapper, 4–54 Zeilen).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `egress_gateway.rs`, `egress_guard.rs`, `egress_vault.rs`, `guarded_payload.rs` | Egress-Klassifikation, typsichere Payloads |
| `entity_recognizer.rs`, `bulk_exfiltration_detector.rs` | Erkennung |
| `processing_registry.rs`, `audit_trace.rs`, `context_edit_audit.rs` | Verarbeitungsverzeichnis, Audit |

## 4. Verträge & Invarianten

1. **Jede externe Wirkung ist budgetiert, auditiert und widerrufbar** (INV-EGRESS-BUDGET-1). Ohne Port ist Egress gesperrt (INV-WIRING-2).
2. Typestate: nur `Sanitized`-Payloads verlassen das System (`GuardedPayload`).
3. Keine Nutzinhalte in Audit-/Logtexten (INV-LOG-1).
4. Kein Rechts-/Löschnachweis hier; das ist `erasure`.

## 5. Ports

**Soll:** implementiert `EgressPolicy`, `EntityRecognizer`, `EgressChannel`, `InjectionDetector` (Sicherheitsports).

## 6. Nebenläufigkeit & Performance

- Hintergrund-Tokio nur in `egress_gateway`; Zustand pro Gateway-Instanz.

## 7. Anti-Patterns

- Egress ohne Klassifikation.
- Standard-Allow-Implementierung.

## 8. Tests & Messung

```bash
cargo test -p contextra-privacy --locked
cargo clippy -p contextra-privacy --all-targets -- -D warnings
```

- 11 Testdateien; Dev-Kante `crypto` (M6 prüfen).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.4 | Port `EgressPolicy` implementieren |
| 3.x | Shredding-Treffer zuordnen |
