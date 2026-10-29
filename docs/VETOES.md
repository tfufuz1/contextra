# Contextra — Feature-Veto-Register (VETOES.md)
<!-- doc-stamp: 2026-10-29 10cc17922 | Spec-Fassung: 3.0 -->
<!--
  WP 0.3 — 2026-10-26:
  - VETO-F02: conditionally_accepted → bestätigt (Überprüfungsfrist 2026-10-07 abgelaufen;
    Tombstone+Slot-Nullung+Rebuild bleibt die einzige zulässige Löschstrategie — AGENTS.md §6)
  - VETO-OP03: conditionally_accepted → bestätigt (kein Audio-Streaming in v1.0 — SPEC §2.3)
  - VETO-F10 und VETO-ADR: unverändert permanent_rejected
-->

## VETO-F02

feature_id: F-02
status: bestätigt
confirmed_at: "2026-10-26"
was: conditionally_accepted
conditional_review_was_due: "2026-10-07"
adr_ref: docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md
affected_paths: ["crates/contextra-vector/", "crates/contextra-store/"]
keywords: ["partial hnsw rebuild", "nucleation", "rebuild_region", "F-02"]
reason: >
  Kein partielles HNSW-Rewiring oder Teilgraph-Rebuilding (F-02) durchführen wegen
  Recall-Kollaps und RwLock-Contention. Zulässig ist ausschließlich:
  Tombstone-Pruning + Slot-Nullung + vollständiger Rebuild (AGENTS.md §6, INV-HNSW-SLOT-1).
  Slot-Nummern werden erst nach Rebuild-Abschluss wiederverwendet.
  Überprüfungsfrist 2026-10-07 ist abgelaufen — Befund B-12 (WP 6.2) bestätigt die Notwendigkeit.

## VETO-F10

feature_id: F-10
status: permanent_rejected
keywords: ["cross-tenant", "osmotic knowledge exchange", "tenant knowledge sharing", "F-10"]
reason: >
  Keine mandantenübergreifenden Datenflüsse, Knowledge-Sharing oder Cross-Tenant-Aggregationen
  (F-10) herstellen. Mandantenisolation (TenantId) ist absolut zur Wahrung von
  DSGVO-Löschgarantien und KV-Cache-Sicherheit. Tenant kommt aus der Transaktion,
  nie aus Nutzerparametern (INV-TENANT-1, SPEC §3 P-Mandant).

## VETO-OP03

feature_id: OP-03
status: bestätigt
confirmed_at: "2026-10-26"
was: conditionally_accepted
conditional_review_was_due: "2026-10-07"
adr_ref: docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md
keywords: ["voice assistant", "speech-to-text", "realtime-audio", "jarvis", "OP-03"]
reason: >
  Keine Realtime-Audio-, Speech-to-Text-, Voice- oder Jarvis-Assistenten-Funktionen (OP-03)
  integrieren. Audio-Streaming gehört nicht zum bi-temporalen Speichersubstrat.
  Kein Audio-Streaming im Kern (SPEC §2.3 Nicht-Ziele v1.0).
  Überprüfungsfrist 2026-10-07 abgelaufen — v1.0-Scope bestätigt kein Audio.

## VETO-ADR

feature_id: VETO-UMGEHUNG
status: permanent_rejected
keywords: ["veto bypass", "isolation bypass", "bypass veto"]
reason: >
  Keine Veto-Sperren oder Isolationsgrenzen ohne explizites ADR unter docs/decisions/ umgehen.
