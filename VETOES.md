# Contextra — Feature-Veto-Register (VETOES.md)

## VETO-F02

feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-10-07
adr_ref: DECISIONS.md#adr-077
keywords: ["partial hnsw rebuild", "nucleation", "rebuild_region", "F-02"]
reason: >
  Kein partielles HNSW-Rewiring oder Teilgraph-Rebuilding (F-02) durchführen wegen Recall-Kollaps und RwLock-Contention; zulässig ist ausschließlich reines Tombstone-Pruning.

## VETO-F10

feature_id: F-10
status: permanent_rejected
keywords: ["cross-tenant", "osmotic knowledge exchange", "tenant knowledge sharing", "F-10"]
reason: >
  Keine mandantenübergreifenden Datenflüsse, Knowledge-Sharing oder Cross-Tenant-Aggregationen (F-10) herstellen; Mandantenisolation (TenantId) ist absolut zur Wahrung von DSGVO-Löschgarantien und KV-Cache-Sicherheit.

## VETO-OP03

feature_id: OP-03
status: conditionally_accepted
conditional_review_due: 2026-10-07
adr_ref: DECISIONS.md#adr-077
keywords: ["voice assistant", "speech-to-text", "realtime-audio", "jarvis", "OP-03"]
reason: >
  Keine Realtime-Audio-, Speech-to-Text-, Voice- oder Jarvis-Assistenten-Funktionen (OP-03) integrieren, da Audio-Streaming nicht zum bi-temporalen Speichersubstrat gehört.

## VETO-ADR

feature_id: VETO-UMGEHUNG
status: permanent_rejected
keywords: ["veto bypass", "isolation bypass", "bypass veto"]
reason: >
  Keine Veto-Sperren oder Isolationsgrenzen ohne explizites ADR in DECISIONS.md umgehen.
