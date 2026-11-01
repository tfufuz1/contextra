# Contextra — Feature-Veto-Register (VETOES.md)

<!-- doc-stamp: 2026-11-01 | Spec-Fassung: 2.0.0 (SOLL-Zustand Release 1.0.0) | Normativ -->

> **Normativer Status:** Dieses Dokument ist nach [`docs/SPEC.md`](SPEC.md) die zweithöchste normative Quelle des Repositories (Rang 2). Hier festgehaltene Vetos dürfen unter keinen Umständen ohne formales Architecture Decision Record (ADR) und Freigabe durch die Core Maintainer aufgehoben oder umgangen werden.

---

## VETO-F02: Kein partielles HNSW-Rewiring

- **Feature-ID:** F-02
- **Status:** Bestätigt (`permanent_enforced`, SPEC §8.10)
- **Betroffene Pfade:** `crates/contextra-vector/`
- **Schlüsselbegriffe:** `partial hnsw rebuild`, `nucleation`, `rebuild_region`, `F-02`
- **Begründung:** Partielles Rewiring oder lokales Re-Graphing im HNSW-Index führt zu Recall-Kollaps, Deadlocks und extremer Lock-Contention. Zulässige Strategie ist ausschließlich: **Tombstone-Markierung + Slot-Zeroing im RAM + vollständiger Batch-Rebuild** ab Erreichen der Schwelle (`vector_rebuild_ratio`). Slot-Nummern dürfen erst nach Zeroing und Snapshot-Rewrite wiederverwendet werden.

---

## VETO-F10: Absolute Mandantentrennung (Zero Cross-Tenant)

- **Feature-ID:** F-10 (SPEC §7.2, §8.8)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Alle Crates (insb. `contextra-engine`, `contextra-store`)
- **Schlüsselbegriffe:** `cross-tenant`, `tenant knowledge sharing`, `TenantPolicy::Optional`, `F-10`
- **Begründung:** Jeglicher mandantenübergreifende Datenfluss, Wissensaustausch oder Cross-Tenant-Index-Sharing ist verboten. Die Mandantenisolation (`TenantId`, `TenantTag`) ist absolut. Zugriff auf den Store erfolgt ausschließlich über `TenantScopedStorage` (T-ARCH-2). `TenantPolicy::Optional` ist restlos gestrichen (D-07).

---

## VETO-SN01: Single-Node-Doktrin (Kein P2P, kein Sharding, kein Server)

- **Feature-ID:** SN-01 (SPEC §1.4, §5.1)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Gesamter Produktionsbaum
- **Schlüsselbegriffe:** `distributed consensus`, `raft`, `p2p`, `sharding`, `replication`, `tcp`, `http-server`
- **Begründung:** Contextra ist ein lokaler, eingebetteter Single-Node-Speicher (Library-First). Ein einziger Primärprozess besitzt und sperrt das lokale Datenverzeichnis (`LOCK`). Verteilte Konsensprotokolle, P2P-Netzwerke oder Cluster-Sharding im Kern erhöhen die Angriffsfläche und untergraben die lokale Löschverifizierbarkeit. Netzwerk-Crates sind im Produktionsbaum verboten (AR-7).

---

## VETO-FAILCLOSED: Keine stillen Fallbacks bei Sicherheitsfehlern

- **Feature-ID:** FC-01 (INV-FAILCLOSED-1)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Alle Schichten
- **Schlüsselbegriffe:** `silent fallback`, `unwrap_or_default`, `ignore error`, `let _ =`, `FC-01`
- **Begründung:** Kryptografische Fehler, fehlende Schlüssel, gebrochene Prüfsummen oder unzureichende Berechtigungen MÜSSEN zu einem sofortigen, expliziten Fehler (`ContextraError`) führen. Es ist strengstens verboten, bei Krypto-Fehlern unverschlüsselte Fallbacks zu wählen, I/O-Fehler mit `let _ =` zu verwerfen oder Löschfehler still zu ignorieren.

---

## VETO-D01: Kein kryptografischer DocDek-Vernichtungspfad in v1 (ADR-001)

- **Feature-ID:** D-01 (ADR-001, SPEC §3.3)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-crypto`, `contextra-store`, `contextra-erasure`
- **Begründung:** Die sogenannte Ebene B (kryptografische Schlüsselvernichtung per `DocDek`/KeyTable) entfällt in v1 vollständig. Löschung in v1 ist die physische Bereinigung (Ebene A: SSTable-Rewrite/Swap, Slot-Zeroing, Segment-Rewrite) kombiniert mit fail-closed `ReadGate`, automatischer Residuenmessung (E1/E2/E3) und signiertem Bericht (`ctx-erasure-report/1`).

---

## VETO-D05: Kein unverschlüsselter Speicherbetrieb

- **Feature-ID:** D-05 (SPEC §10)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-store`, `contextra-engine`, `contextra`
- **Begründung:** Verschlüsselung at rest (AES-256-GCM-SIV) ist in jedem Produktiv-Build Pflicht ohne Opt-Out (`EncryptionMode::Required`). Es gibt keinen unverschlüsselten Block- oder SSTable-Pfad.

---

## VETO-D06: Keine Passphrasen-Ableitung via HKDF

- **Feature-ID:** D-06 (B-15, SPEC §10)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-crypto`
- **Begründung:** Root-Passphrasen dürfen niemals per HKDF-SHA256 abgeleitet werden. Pflicht für Passphrasen ist Argon2id mit Parametern $m \ge 65.536\,\text{KiB}~(64\,\text{MiB}), t \ge 3, p \ge 1$ (`derive_master`). HKDF ist ausschließlich für die Subkey-Ableitung aus $K_\text{master}$ zulässig.

---

## VETO-D27: Keine automatische Entitätsextraktion

- **Feature-ID:** D-27 (SPEC §1.4, §10)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-types`, `contextra-engine`
- **Begründung:** Automatische LLM-basierte Entitätsextraktion (`AutoExtractionMode`) entfällt vollständig in v1, um unbemerkten PII-Abfluss auszuschließen.

---

## VETO-D29: Keine unsafe-Inseln außerhalb von `sys` und `simd`

- **Feature-ID:** D-29 (SPEC §5.2)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Gesamter Workspace
- **Begründung:** `unsafe` Rust-Code ist ausschließlich in `contextra-sys` und `contextra-simd` erlaubt. Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`. `contextra-wire` ist nicht Teil von v1.
