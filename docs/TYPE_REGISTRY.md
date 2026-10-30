# Contextra – Typ-Registry (Kern v1)

<!-- doc-stamp: 2026-10-30 | Normativ: docs/SPEC.md Fassung 4.0 §16 (B7), Regel 11 -->

**Stand:** 30.10.2026. Dieses Dokument ersetzt die frühere Matrix mit über 35 Typen. Die meisten dort genannten Typen gehören zu entfernten Crates (SPEC §5.2) und sind nicht mehr relevant. Der Vorzustand bleibt über Commit `3c1f144fa` erreichbar.

**Zweck:** Regel 11 verlangt eine Typfamilie je Konzept. Hier stehen die öffentlichen Typen, die im Workspace mehrfach definiert sind, mit Zielzustand.

---

## 1. Ermittlung

Die Liste entstand aus folgendem Befehl (Ausgabe am 30.10.2026):

```bash
grep -rhoE "^\s*pub(\(crate\))? (struct|enum|trait|type) [A-Za-z0-9_]+" crates --include=*.rs \
  | awk '{print $NF}' | sort | uniq -d
```

Nach Umsetzung von PLAN WP R1.3 muss die Ausgabe nur noch die in Abschnitt 3 als bewusst gewollt markierten Namen enthalten.

---

## 2. Doppeldefinitionen mit Zielzustand

| Typname | Fundstellen (Ist) | Zielzustand | Maßnahme |
|---|---|---|---|
| `BoundedTopK` | `contextra-text/src/wand.rs`, `contextra-rank/src/fusion/topk.rs` | Eine kanonische Definition | Verschieben nach `contextra-types` oder umbenennen der Textvariante (PLAN R1.3) |
| `HybridQueryBuilder` | `contextra-engine/src/collection/query_builder/builder.rs`, `contextra-types/src/types/saos.rs` | Nur in `contextra-engine` | `types/saos.rs` entfernen (Restmodul) |
| `InMemoryStorageEngine` | `contextra-testkit/src/in_memory_store.rs`, `contextra-store/src/engine/memory_only.rs` | Store-Variante als einzige Implementierung, Testkit re-exportiert | PLAN R1.3 |
| `Signal` | `contextra-types/src/types/saos.rs`, `contextra-rank/src/fusion/types.rs` | Nur in `contextra-rank` | `types/saos.rs` entfernen |

---

## 3. Bewusste Mehrfachnamen

| Typname | Fundstellen | Begründung |
|---|---|---|
| `Result` | `contextra-types/src/error.rs`, `contextra-crypto/src/error.rs` | Typalias je Fehlerdomäne, kein Konflikt in Modulen |
| `DocId`, `DocIdRaw` | `contextra-types/src/types/domain/ids.rs`, `contextra-text/src/posting_list.rs` | Verschiedene Konzepte: Domänen-ID und interne Postings-ID |
| `BlockCacheShard` | `contextra-store/src/sstable/block_cache.rs` | Typalias, cfg-abhängig über Feature `block-cache-v2` (zwei Alias-Ziele, nicht zwei Definitionen) |

---

## 4. Pflege

- Neue öffentliche Typen zuerst über die Suche prüfen. Existiert ein kanonischer Typ, wird er importiert.
- Wird ein Typ dupliziert, muss der PR diesen Eintrag in Abschnitt 2 ergänzen oder die Duplikation beseitigen.
