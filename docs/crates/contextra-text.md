# contextra-text — Crate-Spezifikation

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Bindung:** `docs/SPEC.md` §2, §3, §5, §6, §7

---

## 1 Zweck & Domäne

`contextra-text` ist der invertierte Volltextindex des Baukastens:
- Deutsche Morphologie, Tokenizer, Lemmatisierung, Zerlegung von Komposita
- Posting-Listen mit Block-Max-WAND (BMW), BM25 und BM25F Relevanzbewertung
- Persistente Indizierung über `PostingStore`-Abstraktion

## 2 Datenklasse & DSGVO-Rolle

- **Datenklasse D (Ableitung):** Der Volltextindex enthält Terme, Tokens und Doc-IDs, die aus Quelltexten abgeleitet sind.
- Da Terme personenbezogene Daten widerspiegeln können (z.B. Namen, Kontaktdaten), unterliegt `contextra-text` der lückenlosen Bereinigungspflicht.

## 3 Löschkette & ErasureLayer-Implementierung

`contextra-text` implementiert das Trait `ErasureLayer` mit statischer `LayerId`:

1. **`resolve(target)`:** Identifiziert die betroffenen Segmente und Posting-Listen für die gelöschten Dokumente.
2. **`on_gate(scope)`:** Markiert die Dokument-IDs in einer In-Memory-Ausschlussmaske (`RoaringBitmap`), damit Suchanfragen das Dokument ab sofort ignorieren.
3. **`purge(scope, cx)` (Priorität 4b — Text-Rewrite):** 
   - Führt ein deterministisches Segment-Rewrite durch.
   - Entfernt gelöschte Dokumente physisch aus den invertierten Listen.
   - Bereinigt Term-Frequenzen und Dokumentlängenstatistiken.
   - Schreibt die bereinigten Segmente atomar neu und synchronisiert via `durable-fs`.
4. **`witness(scope, scanner)`:** 
   - Verifiziert im Read-Only-Modus, dass keine Posting-Listen oder Term-Positionen mehr für die Ziel-IDs existieren.
   - Gibt `residue_matches == 0` zurück.

## 4 Lesesperre (ReadGate)

- Suchanfragen prüfen vor oder während der Block-Max-WAND-Evaluation gegen den synchronen `ReadGate`.
- Dokumente im Zustand `Erased` oder `Restricted` werden vor der BM25-Kandidatenauswertung ausgefiltert.

## 5 Invarianten

- `#![forbid(unsafe_code)]`.
- Deterministische Term-Reihenfolge und Tie-Breaking bei gleicher Relevanz.
- Keine stillen Fallbacks oder ungesicherten Caches.
