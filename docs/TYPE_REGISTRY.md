# Contextra – Typ-Registry (Baukasten-System)

<!-- doc-stamp: 2026-11-08 | Normativ: docs/SPEC.md Fassung 4.1 §2 (B3, B4), §3.1, §15 (E-02) -->

**Stand:** 2026-11-08.
Dieses Dokument dokumentiert die Typen-Landschaft, Prinzipien der Typentkopplung und das Monitoring von Namenskollisionen im modularen **Contextra Baukastensystem**.

---

## 1. Leitprinzipien der Baukasten-Typisierung (SPEC v4.1)

1. **Vertikale Autarkie vor DRY (§3.1):**
   Wiederholungen trivialer Datenstrukturen (z. B. `DistanceMetric`, `ScoredPoint`, `Token`, `SliceRange`) zwischen voneinander unabhängigen Crates sind erwünscht und normativ vorgeschrieben. DRY über Crate-Grenzen hinweg erzeugt unzulässige Kopplung.
2. **Standardtypen in öffentlichen APIs (B3):**
   Öffentliche APIs von Klasse-A-Bausteinen verwenden Standard-Rust-Typen (`&[f32]`, `&[u8]`, `&str`, `u64`, `usize`) oder Crate-eigene Typen.
3. **Lokales Fehlermodell (B4):**
   Jedes Crate definiert genau einen primären Fehlertyp (`Error`, `thiserror`, `#[non_exhaustive]`). Kein globaler Fehler-Hub.
4. **Keine Hub-Abhängigkeit zu `types` in Klasse A (E-02):**
   Klasse-A-Bausteine importieren weder `contextra-types` noch `contextra-ports`. Typ-Konvertierungen erfolgen in Adaptern oder in der Integrationsschicht (Klasse C).

---

## 2. Kanonische Zuordnungen & Kollisionsüberwachung

Zur Vermeidung von Verwirrung und unbeabsichtigtem Shadowing wird die Verwendung öffentlicher Namen über Crates hinweg auditiert:

| Typname | Crate(s) | Status & Vorgabe |
|---|---|---|
| `DistanceMetric` | `contextra-simd` (kanonisch für Kernel), lokale Enums in Adaptern | Vertikal autark; keine Abhängigkeit von `simd` auf `types` |
| `BoundedTopK` | `contextra-rank` (kanonisch für Rangfusion), `contextra-text` | Lokal getrennt gekapselt |
| `HybridQueryBuilder` | `contextra-engine` | Exklusiv in der Integrationsschicht |
| `InMemoryStorageEngine` | `contextra-testkit` | Exklusiv in Tooling / Testdoubles |
| `Error` / `Result` | Jedes Crate lokal | Erlaubte und vorgeschriebene Crate-eigene Fehlerdomäne (B4) |

---

## 3. Typ-Audits

Das Tooling prüft Typkollisionen und Schnittstellenreinheit automatisiert:
```bash
python3 tools/type_registry_sync.py
```
Ergebnisse werden in `docs/diagnostics/TYPE_REGISTRY_COLLISIONS.md` dokumentiert.
