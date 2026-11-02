# Contextra v1 – Architektur-Review & Technische Entscheidungsvorlage

> **Basis:** SPEC 4.3.0 · HEAD `909ff27` · Stand 2026-11-02
> **Prüfmodus:** Quelltext gelesen, `cargo check` ausgeführt, Manifeste und Backlog ausgewertet.
> **Normative Quelle:** Ausschließlich [`docs/SPEC.md`](docs/SPEC.md). Dieses Dokument ist abgeleitet (D-98).

---

## Executive Summary

Contextra verfolgt eine architektonisch tragfähige Grundidee: Crypto-Shredding mit hierarchischem
Schlüsselbaum (SPEC 10.1), LSM-Löschcluster-Schlüsselraum (D-31) und einem signierten,
unabhängig prüfbaren Bericht (D-13). Die SPEC 4.3.0 ist in sich konsistent und schließt die vier
strukturellen Lücken der Vorfassungen (P11, Schlüsselabdeckung, Messbarkeit, O(1)-Zielgrößenunabhängigkeit).

**Der Build bricht an HEAD** – `contextra-types` schlägt mit Fehlern fehl,
die den gesamten Workspace blockieren. Defekte WP-0.3 (B-1 bis B-3) sowie S-1 bis S-4
(Sicherheits-Noncompliance) werden durch diesen PR behoben.

---

## 1. Ist-Zustand je Komponente (Stand Prüfung)

### Ring 0–1 (Fundament)

| Crate | Status | Befund |
|---|---|---|
| `contextra-types` | ❌ baut nicht | Zeroize-Derive-Makros fehlen, `[u8; 65]`/`[u8; 78]` Serde-Fehler (B-1) |
| `contextra-ports` | ✅ baut | Löschports `ErasureLayer`, `ReadGate` vorhanden |
| `contextra-sys` | ✅ baut | Prozesshärtung, fs_class, POSIX-Primitiven |
| `contextra-crypto` | ✅ baut | AES-256-GCM-SIV, Argon2id, HKDF, Ed25519, `KeyHierarchy` |
| `contextra-mvcc` | ✅ baut | Sentinel-Fix done (WP-3.3-mvcc), `SnapshotRegistry` mandantengetrennt |
| `contextra-simd` | ✅ baut | f32-Kernel, VNNI entfernt |
| `contextra-durable-fs` | ✅ baut | `RealVfs` auf `ports::Vfs` umgestellt |

### Ring 2 (Store)

**Build-Fehler (F-32, behoben durch diesen PR):**

- `store/src/lsm/config.rs` – `contextra_ports::license` nicht gefunden (B-2, LicenseGate-Relikt)
- `store/src/wal/replay.rs` – Syntaxfehler (`mmap` nach Buffer-Umstellung)
- `store/src/wal/mod.rs` – `allow_legacy_integrity_key_fallback` Laufzeit-Downgrade (S-2, D-05)

**Sicherheits-Noncompliance (S-1 bis S-4, behoben):**

- `Option<Arc<KeyManager>>` im WAL → Pflichtfeld (D-05, F-09)
- `DurabilityMode::WalNoHmac` → entfernt (D-05)
- `allow_legacy_integrity_key_fallback` → entfernt (D-05, F-21)
- `LsmConfig::encryption_passphrase: Option<String>` → entfernt (D-05, F-09)
- `tokio` mit `features = ["full"]` → auf `rt-multi-thread,sync,time,macros` begrenzt (D-54, S-4)

### Ring 3 (Erasure)

`contextra-erasure` ist als Skelett angelegt – State-Machine, Gate, Journal, Vault, Messung sind
typentheoretisch modelliert. Vollimplementierung: WP-4.2/4.3.

---

## 2. Beantworte Architekturfragen (SPEC-normativ)

### 2.1 Crash-Resilienz & Saga-Pattern

Die Saga-State-Machine (`Requested → Gated → KeyDestroyed → Purging → Verifying → Completed`)
ist korrekt modelliert. Der normative Normalpfad (D-58) umfasst 4 `fsync` je Anfrage:

```
1. Journal[Requested+Intent]  → fsync   (I-8: vor Gate dauerhaft)
2. Gate-Log[erase_id, range]  → fsync   (I-2: vor Lesepfaden geladen)
3. Vault: Slot nullen          → sync_data (I-9: vor DEK)
4. Journal[KeyDestroyed]      → fsync   → Rückkehr mit Gated
```

Fehlerszenarien FM-1..FM-17 (SPEC 13.2) sind spezifiziert. Recovery-Logik (SPEC 14.1,
Schritt 7) ist im Saga-Treiber (WP-4.2) zu verdrahten.

### 2.2 Concurrency & Lock-Contention (D-57)

Kein Document-Lock-Set bei Subject/Collection/Tenant-Erase. Stattdessen:
Commit-Barriere (O(1), `gated_seq`), ReadGate-Check im Gruppen-Commit-Leader.
`SnapshotRegistry` mandantengetrennt (D-61). Kompaktion bricht kooperativ ab (I-16, max. 1 s).

### 2.3 Hardware-Abstraktion & Wear-Leveling

Garantien oberhalb der `Vfs`-Schicht: kryptografische Unlesbarkeit (DEK-Vernichtung O(1)),
physische Entfernung (SSTable-Rewrite + `unlink`), Dateischlüssel-Vernichtung (F-Slot, D-85).
Restrisiken `WearLevelingRemanence`, `FsJournalOrCow` sind im signierten Bericht (D-13) benannt.
NIST SP 800-88 Rev. 2 Cryptographic Erase ist Stand der Technik. Startprobe (D-56) verweigert
Start auf unbekannten Dateisystemen.

### 2.4 Roadmap S0 (kritischer Pfad)

```
WP-0.3 (Build) → WP-1.1 (Rückbau) → WP-1.4 (Vfs) → WP-2.1 (Engine)
→ WP-3.1 (Types) → G0 (Benchmark) → WP-3.3 (Store)
→ WP-4.1 (Vault) → WP-4.2 (Gate+Saga) → WP-4.3 (Messung)
→ WP-5.1 (Engine-Integration) → Gate G-D
```

---

## 3. Behobene Defekte (dieser PR)

### Stufe 0: Build-Blockierer

| ID | Defekt | Datei | Behoben durch |
|---|---|---|---|
| B-1 | `zeroize`-Derive-Makros fehlen in `types` | `types/Cargo.toml`, `types/src/secret.rs` | Feature `zeroize_derive` aktiviert |
| B-1b | `[u8; 65]`/`[u8; 78]` nicht von `serde::Serialize` | `types/src/key.rs` | Manuelles `Serialize`/`Deserialize` impl |
| B-2 | `contextra_ports::license` nicht gefunden | `store/src/lsm/config.rs` | Import entfernt |
| B-3 | Syntaxfehler `mmap` nach Buffer-Umstellung | `store/src/wal/replay.rs` | Variable korrigiert |

### Stufe 1: Sicherheits-Noncompliance (D-05, P5)

| ID | Defekt | Behoben durch |
|---|---|---|
| S-1 | `Option<Arc<KeyManager>>` im WAL | `key_manager` ist jetzt `Arc<KeyManager>` (nicht optional) |
| S-2 | `allow_legacy_integrity_key_fallback` Laufzeit-Downgrade | Feld + Logik entfernt (D-05, F-21) |
| S-2b | `DurabilityMode::WalNoHmac` abschwächbar | Variante entfernt |
| S-3 | `LsmConfig::encryption_passphrase: Option<String>` | Feld entfernt (Verschlüsselung über `KeyManager`, D-05) |
| S-4 | `tokio` mit `features = ["full"]` | Workspace-Manifest auf `rt-multi-thread,sync,time,macros` begrenzt (D-54) |

---

## 4. Offene Arbeitspakete (nicht in diesem PR)

| WP | Titel | Gate | Status |
|---|---|---|---|
| WP-1.1 | Graph-Rückbau, Altlasten, `AutoExtractionMode` | B | todo |
| WP-1.4 | 169 direkte `std::fs`-Aufrufe in `store` auf `Vfs` | B | in_progress |
| WP-1.5 | Struktur, Namen, Crate-Köpfe, Sitzungsmarken | B | todo |
| WP-2.1 | `engine` wieder kompilierbar | A | todo |
| WP-3.1 | `DocRef`, Löschtypen in `types` | G0 | in_progress |
| WP-4.2 | Gate-Log, Saga-Treiber, Recovery | G-D | todo |
| WP-4.3 | Messung, Bericht, Ledger | G-D | todo |
| WP-5.1 | Engine: ReadGate im Commit, DocRef-API | G-D | todo |

---

## 5. Regulatorischer Kontext

Das neue **ProdHaftG** gilt für Produkte, die ab **09.12.2026** in Verkehr gebracht werden.
**CRA-Meldepflichten** (Art. 14) gelten seit **11.09.2026**. Enterprise-Testdeployments vor
dem S0-Release sind explizit als „nicht in Verkehr gebracht" zu dokumentieren (E-L9, D-67).

Die SPEC-Formulierung (6.4): *„Contextra macht Löschen zu einem messbaren, unabhängig
prüfbaren Vorgang: Wirkung sofort, Entfernung physisch, Nachweis signiert, Restrisiken benannt."*
ist die einzige rechtskonform ehrliche Produktaussage.

---

*Abgeleitet von SPEC 4.3.0. Bei Widerspruch gilt die SPEC (D-98).*
