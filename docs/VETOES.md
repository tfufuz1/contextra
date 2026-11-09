# Contextra — Feature-Veto-Register (VETOES.md)
<!-- doc-stamp: 2026-11-08 | Spec-Fassung: 4.1 -->
<!--
  Normativ verankert in docs/SPEC.md Fassung 4.1.
  Sichert unverletzliche Architektur- und Sicherheitsgrenzen im Baukastensystem.
-->

## VETO-F02

feature_id: F-02
status: bestätigt
confirmed_at: "2026-11-08"
adr_ref: docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md
affected_paths: ["crates/contextra-vector/", "crates/contextra-store/"]
keywords: ["partial hnsw rebuild", "nucleation", "rebuild_region", "F-02"]
reason: >
  Kein partielles HNSW-Rewiring oder unkontrolliertes Teilgraph-Rebuilding (F-02) im Kern durchführen wegen
  Gefahr von Recall-Kollaps und RwLock-Contention. Zulässig ist ausschließlich:
  Tombstone-Pruning + Slot-Nullung (`INV-VEC-ZEROIZE-1`) und inkrementeller Neuaufbau
  unter Wahrung der Konnektivität (`INV-VEC-CONNECTIVITY-1`, `docs/SPEC.md` §11.4).
  Slot-Nummern werden erst nach gesicherter Reparatur wiederverwendet.

## VETO-F10

feature_id: F-10
status: permanent_rejected
keywords: ["cross-tenant", "osmotic knowledge exchange", "tenant knowledge sharing", "F-10"]
reason: >
  Keine mandantenübergreifenden Datenflüsse, Knowledge-Sharing oder Cross-Tenant-Aggregationen
  (F-10) herstellen. Mandantenisolation (TenantId) ist absolut zur Wahrung von
  Löschgarantien und KV-Cache-Sicherheit. Die TenantId stammt aus dem autorisierten Kontext,
  niemals aus unsicheren Nutzerparametern (`docs/SPEC.md` §3, §12.2, §14.1).

## VETO-OP03

feature_id: OP-03
status: bestätigt
confirmed_at: "2026-11-08"
adr_ref: docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md
keywords: ["voice assistant", "speech-to-text", "realtime-audio", "jarvis", "OP-03"]
reason: >
  Keine Realtime-Audio-, Speech-to-Text-, Voice- oder Jarvis-Assistenten-Funktionen (OP-03)
  in den Speicherkern oder die Basis-Bausteine integrieren. Audio-Streaming gehört nicht zum
  Aufgabenbereich der RAG-, Vektor- und Persistenzbausteine (`docs/SPEC.md` §0, §2).

## VETO-ADR

feature_id: VETO-UMGEHUNG
status: permanent_rejected
keywords: ["veto bypass", "isolation bypass", "bypass veto"]
reason: >
  Keine Veto-Sperren, Klassengrenzen (Klasse A B1–B12) oder Isolationsgrenzen ohne
  explizites ADR unter docs/decisions/ und Freigabe im normativen Entscheidungsregister
  (`docs/SPEC.md` §15) umgehen.
