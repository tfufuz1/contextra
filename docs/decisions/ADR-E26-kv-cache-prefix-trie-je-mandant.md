# ADR-E26: kv-cache-prefix-trie-je-mandant

* **Status:** Akzeptiert (aus Spezifikation §13 E26)
* **Datum:** 2026-10-14

## Entscheidung
KV-Cache: Prefix-Trie je Mandant; Refcount-Pin ≤ 50 %; SnapKV nur mit candle; Rückholung `O(Block)` (§9.4).

## Begründung
Timing-Seitenkanal; Eviction-Blockade; Attention-Zugriff nur in candle.

## Verworfen
Mandantenübergreifender Trie; „O(1)"
