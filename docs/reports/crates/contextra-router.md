# Audit- & Messreport: `contextra-router`

- **Crate-Pfad:** `crates/contextra-router`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 199 bestanden, 0 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3478s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (12):** `shlex`, `contextra-types`, `contextra-wire`, `contextra-ports`, `contextra-adapt`, `arc-swap`, `parking_lot`, `tokio`, `serde`, `serde_json`, `thiserror`, `tracing`
- **Features (8):** `default`, `subprocess-dispatch`, `bandit-routing`, `cloud-egress-guard`, `egress-sherman-morrison`, `flow-corrected-thompson`, `exp-fc-ts`, `exp-routing-strategy`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.493s
- **Tests bestanden:** 85
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 114 | 0 | 0.5347s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.5228s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `router_bench` | ✅ OK | **0.85s** | 4 Pfade |

---|---|---|
| `router_bench` | ⏱️ TIMEOUT | 20.0286s |
| `bandit_latency_gate` | ⏱️ TIMEOUT | 20.0287s |
| `cost_routing_savings_bench` | ⏱️ TIMEOUT | 20.0288s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
