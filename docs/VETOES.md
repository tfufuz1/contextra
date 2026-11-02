# Contextra — Feature-Veto-Register (VETOES.md)

<!-- doc-stamp: 2026-11-02 | Spec-Fassung: 4.0.0 (SOLL-Zustand Release 1.0.0) | Normativ abgeleitet -->

> **Normativer Status:** Dieses Dokument registriert verbindliche Architektur- und Design-Vetos abgeleitet aus [`docs/SPEC.md`](SPEC.md). Hier festgehaltene Vetos dürfen unter keinen Umständen ohne formales Architecture Decision Record (ADR) und Freigabe durch die Core Maintainer aufgehoben oder umgangen werden.

---

## VETO-D01b: Drei Säulen der Löschung in v1 (Schlüsselvernichtung Pflicht)

- **Feature-ID:** D-01 (ADR-002, ersetzt ADR-001 / VETO-D01; SPEC §1.1, §4.1 D-01, §11)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `crates/contextra-erasure/`, `crates/contextra-crypto/`, `crates/contextra-store/`, `crates/contextra-types/`
- **Schlüsselbegriffe:** `key destruction`, `file vault`, `crypto shredding`, `three pillars of erasure`, `keyslots`, `D-01`
- **Begründung:** Das frühere VETO-D01 („keine Ebene B in v1“) ist durch ADR-002 und SPEC D-01 aufgehoben. Löschung in v1 ruht zwingend auf drei Säulen: (1) Gate und physischer Purge, (2) kryptografische Schlüsselvernichtung im hierarchischen Tresor (`T`, `C`, `S`, `D`-Slots gem. D-79 sowie `F`-Dateischlüssel gem. D-85), (3) unabhängige Messung (MK1–MK3) und signierter Bericht. Das Datenformat lässt sich nach Auslieferung nicht rückwirkend ändern.

---

## VETO-F02: Kein unkoordiniertes partielles HNSW-Rewiring ohne Surrogat & Rebuild

- **Feature-ID:** F-02 (SPEC §4.1 D-18, §4.3 D-77, §8.10)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `crates/contextra-vector/`
- **Schlüsselbegriffe:** `partial hnsw rebuild`, `nucleation`, `rebuild_region`, `F-02`
- **Begründung:** Unkoordiniertes partielles Rewiring führt zu Recall-Kollaps und Lock-Contention. Das zulässige Verfahren ist strikt dreistufig nach D-77:
  1. *Sofort vor `Gated`:* Ersetzen der Zielvektoren in allen Repräsentationen durch ein Navigationssurrogat (Mittelwert nicht-gegateter Nachbarn bzw. Einstiegsknoten; kein Zielvektorbyte verbleibt in der Arena).
  2. *Im Purge-Fenster:* Batch-Kantenreparatur in einem linearen Durchlauf über alle Adjazenzlisten; Entfernen der Kanten auf Zielslots, Neuverlinkung aus Nachbarn, Freigabe der Slots.
  3. *Deterministischer Rebuild:* Bei Überschreiten von `vector_rebuild_ratio` (0,2) oder ≥ 10 % reparaturgelöschter Slots deterministischer Vollrebuild in `doc_tag`-Reihenfolge.

---

## VETO-F10: Absolute Mandantentrennung (Zero Cross-Tenant)

- **Feature-ID:** F-10 (SPEC §4.1 D-07, §5.2 AR-6, §7.2, §8.8)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Alle Crates (insb. `contextra-engine`, `contextra-store`)
- **Schlüsselbegriffe:** `cross-tenant`, `tenant knowledge sharing`, `TenantPolicy::Optional`, `F-10`
- **Begründung:** Jeglicher mandantenübergreifende Datenfluss, Wissensaustausch oder Cross-Tenant-Index-Sharing ist verboten. Die Mandantenisolation (`TenantId`, `TenantTag`) ist absolut. Zugriff auf den Store erfolgt ausschließlich über `TenantScopedStorage` (T-ARCH-2). Mandantenlose Zugriffsfunktionen und `TenantPolicy::Optional` sind restlos gestrichen (D-07).

---

## VETO-SN01: Single-Node-Doktrin (Kein P2P, kein Sharding, kein Server)

- **Feature-ID:** SN-01 (SPEC §1.1, §1.4, §5.1, §5.2 AR-7)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Gesamter Produktionsbaum
- **Schlüsselbegriffe:** `distributed consensus`, `raft`, `p2p`, `sharding`, `replication`, `tcp`, `http-server`, `socket`
- **Begründung:** Contextra ist eine lokale, eingebettete Single-Node-Bibliothek (Library-First). Ein einziger Primärprozess besitzt und sperrt das lokale Datenverzeichnis (`LOCK`). Verteilte Konsensprotokolle, P2P-Netzwerke oder Cluster-Sharding im Kern erhöhen die Angriffsfläche und untergraben die lokale Löschverifizierbarkeit. Netzwerk-Crates sind im Produktionsbaum verboten (AR-7, D-54).

---

## VETO-FAILCLOSED: Keine stillen Fallbacks bei Sicherheitsfehlern

- **Feature-ID:** FC-01 (P2, INV-FAILCLOSED-1, SPEC §2)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Alle Schichten
- **Schlüsselbegriffe:** `silent fallback`, `unwrap_or_default`, `ignore error`, `let _ =`, `FC-01`
- **Begründung:** Kryptografische Fehler, fehlende Schlüssel, gebrochene Prüfsummen oder unzureichende Berechtigungen MÜSSEN zu einem sofortigen, expliziten Fehler (`ContextraError`) führen. Es ist verboten, bei Krypto-Fehlern unverschlüsselte Fallbacks zu wählen, I/O-Fehler mit `let _ =` zu verwerfen oder Löschfehler still zu ignorieren.

---

## VETO-D05: Kein unverschlüsselter Speicherbetrieb & kein Integritäts-Downgrade

- **Feature-ID:** D-05 (SPEC §4.1 D-05, §4.3 D-84, §10)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-store`, `contextra-engine`, `contextra-crypto`
- **Begründung:** Verschlüsselung at rest (AES-256-GCM-SIV) und WAL-HMAC-Integrität sind in jedem Produktiv-Build Pflicht ohne Opt-Out. Es gibt keinen unverschlüsselten Block- oder SSTable-Pfad. Feature-Flags wie `legacy-wal-key`, `encryption-at-rest` oder Parameter wie `allow_legacy_integrity_key_fallback` sind verboten.

---

## VETO-D06: Keine Passphrasen-Ableitung via HKDF

- **Feature-ID:** D-06 (SPEC §4.1 D-06, §10.1)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-crypto`
- **Begründung:** Root-Passphrasen dürfen niemals per HKDF abgeleitet werden. Pflicht für Passphrasen ist Argon2id mit strengen Grenzen: $m \in [64\,\text{MiB}, 4\,\text{GiB}], t \in [3, 16], p \in [1, 16]$. HKDF ist ausschließlich für die Subkey-Ableitung aus $K_\text{master}$ bzw. $K_\text{unit}$ zulässig.

---

## VETO-D27: Keine automatische Entitätsextraktion im Kern

- **Feature-ID:** D-27 (SPEC §1.4, §4.1 D-24)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** `contextra-types`, `contextra-engine`
- **Begründung:** Automatische Entitäts- oder Metadatenextraktion, Textparsing und ML-Inferenz entfallen im Kern vollständig, um unbemerkten PII-Abfluss auszuschließen. Text und Vektoren werden vom Aufrufer übergeben.

---

## VETO-D29: Keine unsafe-Inseln außerhalb von `sys` und `simd`

- **Feature-ID:** D-29 (SPEC §5.2 AR-8, §8.3, §8.4)
- **Status:** Dauerhaft bestätigt (`permanent_enforced`)
- **Betroffene Pfade:** Gesamter Workspace
- **Begründung:** `unsafe` Rust-Code ist ausschließlich in `contextra-sys` (POSIX-Plattformprimitive) und `contextra-simd` (f32-Distanzkernel) erlaubt. Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`.
