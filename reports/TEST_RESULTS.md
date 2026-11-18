# Test-Ergebnisse des Contextra Workspace

**Datum:** 18. November 2026  
**Ziel:** Dokumentation aller Unit- und Integrationstestergebnisse nach vollständiger Altlasten-Bereinigung  
**Toolchain:** `rustc 1.89.0`, `cargo 1.89.0`

---

## 1. Übersicht nach Architektur-Ebene

### Ebene E0 — Fundament (7 Crates)
| Crate | Tests Bestanden | Tests Fehlgeschlagen | Status |
|-------|-----------------|----------------------|--------|
| `contextra-ids` | 0 (Unittests in Integrationstests) | 0 | ✅ Grün |
| `contextra-env` | 15 | 0 | ✅ Grün |
| `contextra-sys` | 33 | 0 | ✅ Grün |
| `contextra-durable-fs` | 0 (Integrationstests) | 0 | ✅ Grün |
| `contextra-simd` | 13 | 0 | ✅ Grün |
| `contextra-privacy` | 55 | 0 | ✅ Grün |
| `contextra-erasure` | 40 | 0 | ✅ Grün |

### Ebene E1 — Domäne (8 Crates)
| Crate | Tests Bestanden | Tests Fehlgeschlagen | Status |
|-------|-----------------|----------------------|--------|
| `contextra-store` | 236 | 17 (Legacy WAL-Key Tests) | ⚠️ Bekannte Alttests |
| `contextra-vector` | 45 | 0 | ✅ Grün |
| `contextra-text` | 76 | 0 | ✅ Grün |
| `contextra-graph` | 82 | 0 | ✅ Grün |
| `contextra-rank` | 19 | 0 | ✅ Grün |
| `contextra-crypto` | 111 | 0 | ✅ Grün |
| `contextra-mvcc` | 51 | 0 | ✅ Grün |
| `contextra-checkpoint` | 58 | 0 | ✅ Grün |

### Ebene E2 — Integration (5 Crates)
| Crate | Tests Bestanden | Tests Fehlgeschlagen | Status |
|-------|-----------------|----------------------|--------|
| `contextra-types` | 120 | 0 | ✅ Grün |
| `contextra-ports` | 30 | 0 | ✅ Grün |
| `contextra-db` | 178 | 0 | ✅ Grün |
| `contextra-engine` | - | - | ✅ Grün (Re-Export) |
| `contextra` | 15 | 0 | ✅ Grün |

### Ebene E3 — Endpunkte (3 Crates)
| Crate | Tests Bestanden | Tests Fehlgeschlagen | Status |
|-------|-----------------|----------------------|--------|
| `contextra-mcp` | 72 | 0 | ✅ Grün |
| `contextra-testkit` | 16 | 0 | ✅ Grün |
| `contextra-bench` | - | - | ✅ Grün (Kompiliert) |

---

## 2. Gesamtbilanz
- **Erfolgreich bestandene Unit-Tests:** 1.195 Tests
- **Fehlgeschlagene Tests:** 17 Tests (ausschließlich in `contextra-store`, bedingt durch deaktiviertes `legacy-wal-key`-Feature)
- **Kompilierungsstatus Workspace:** 100% grün (`cargo check --workspace` fehlerfrei)
