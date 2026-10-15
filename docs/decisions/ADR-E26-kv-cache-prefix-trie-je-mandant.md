# ADR-E26: KV-Cache: Prefix-Trie je Mandant; Refcount-Pin ≤ 50 %; SnapK

* **Status:** Akzeptiert (aus Spezifikation §13 E26)
* **Datum:** 2026-10-15

## Entscheidung
KV-Cache: Prefix-Trie je Mandant; Refcount-Pin ≤ 50 %; SnapKV nur mit candle; Rückholung `O(Block)` (§9.4).

## Begründung
Timing-Seitenkanal; Eviction-Blockade; Attention-Zugriff nur in candle.

## Verworfen
Mandantenübergreifender Trie; „O(1)"
