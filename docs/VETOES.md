# Contextra — Feature-Veto-Register (VETOES.md)

<!-- doc-stamp: 2026-10-31 | Spec-Fassung: 2.4 | Normativ -->

> **Normativer Status:** Dieses Dokument ist nach [`docs/SPEC.md`](SPEC.md) die zweithöchste normative Quelle des Repositories (Rang 2). Hier festgehaltene Vetos dürfen unter keinen Umständen ohne formales Architecture Decision Record (ADR) und Freigabe durch die Core Maintainer aufgehoben oder umgangen werden.

---

## VETO-F02: Kein partielles HNSW-Rewiring

- **Feature-ID:** F-02
- **Status:** Bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `crates/contextra-vector/`, `crates/contextra-store/`
- **Schlüsselbegriffe:** `partial hnsw rebuild`, `nucleation`, `rebuild_region`, `F-02`
- **Begründung:** Partielles Rewiring oder lokales Re-Graphing im HNSW-Index führt zu Recall-Kollaps, Deadlocks und extremer RwLock-Contention. Zulässige Strategie ist ausschließlich: **Tombstone-Pruning + Slot-Nullung + vollständiger Rebuild** (INV-HNSW-SLOT-1). Slot-Nummern dürfen erst nach Abschluss des vollständigen Rebuilds wiederverwendet werden.

---

## VETO-F10: Absolute Mandantentrennung (Zero Cross-Tenant)

- **Feature-ID:** F-10
- **Status:** Dauerhaft abgelehnt (`permanent_rejected`)
- **Betroffene Pfade:** Alle Crates (insb. `contextra-db`, `contextra-engine`, `contextra-mcp`)
- **Schlüsselbegriffe:** `cross-tenant`, `osmotic knowledge exchange`, `tenant knowledge sharing`, `F-10`
- **Begründung:** Jeglicher mandantenübergreifende Datenfluss, Wissensaustausch oder Cross-Tenant-Index-Sharing ist verboten. Die Mandantenisolation (`TenantId`) ist absolut zur Wahrung von Art. 17 DSGVO und vertraglichen Geheimhaltungspflichten. `TenantId` stammt unveränderlich aus dem `RequestContext`, niemals aus Nutzereingaben oder Parametern (INV-TENANT-1, SPEC §3 P-Mandant).

---

## VETO-OP03: Kein Audio-, Voice- oder Speech-Streaming

- **Feature-ID:** OP-03
- **Status:** Bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Gesamter Kern
- **Schlüsselbegriffe:** `voice assistant`, `speech-to-text`, `realtime-audio`, `jarvis`, `OP-03`
- **Begründung:** Realtime-Audio, STT/TTS oder Sprachassistenten-Schnittstellen gehören nicht zum Funktionsumfang eines lokalen Wissens- und Speichersubstrats (SPEC §2.3 Nicht-Ziele v1.0).

---

## VETO-SN01: Single-Node-Doktrin (Kein P2P, kein Sharding)

- **Feature-ID:** SN-01
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-store`, `contextra-engine`, `contextra-core`
- **Schlüsselbegriffe:** `distributed consensus`, `raft`, `p2p`, `sharding`, `replication`
- **Begründung:** Contextra ist ein lokaler, eingebetteter Single-Node-Speicher. Ein einziger Primärprozess besitzt und sperrt das lokale Datenverzeichnis. Verteilte Konsensprotokolle, P2P-Netzwerke oder Cluster-Sharding im Kern erhöhen die Angriffsfläche und untergraben die lokale Löschverifizierbarkeit.

---

## VETO-FAILCLOSED: Keine stillen Fallbacks bei Sicherheitsfehlern

- **Feature-ID:** FC-01
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Alle Schichten
- **Schlüsselbegriffe:** `silent fallback`, `unwrap_or_default`, `ignore error`, `FC-01`
- **Begründung:** Kryptografische Fehler, fehlende Schlüssel, gebrochene Prüfsummen oder unzureichende Berechtigungen MÜSSEN zu einem sofortigen, expliziten Fehler (`ContextraError`) führen. Es ist strengstens verboten, bei Krypto-Fehlern unverschlüsselte Fallbacks zu wählen oder Löschfehler still zu ignorieren (INV-FAILCLOSED-1).

---

## VETO-ADR: Verbot der Veto-Umgehung

- **Feature-ID:** VETO-UMGEHUNG
- **Status:** Dauerhaft abgelehnt (`permanent_rejected`)
- **Begründung:** Kein Veto darf durch modulinterne Hacks, unautorisierte Abkürzungen oder Refactorings umgangen werden. Jede Änderung erfordert ein formelles ADR unter `docs/decisions/`.
