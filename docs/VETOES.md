# Veto-Register

Abgeleitet aus [`SPEC.md`](SPEC.md). Bei Widerspruch gilt die SPEC. Ein vorhandenes Veto darf nur durch eine SPEC-Änderung mit Eintrag in Anhang H aufgehoben werden.

Diese Fassung ersetzt die Fassung 2.0.0. Dabei wurden die Vetos gegen die SPEC-Entscheidungen 4.x abgeglichen (Anhang E, KF-1).

---

## VETO-SN01: Single-Node, kein Server, kein Netzwerk

- **Quelle:** SPEC 1.4, HR-1, HR-2, D-43, D-54
- **Status:** gültig
- **Regel:** Keine Netzwerkfunktion, kein Server, kein Socket, kein P2P, kein Sharding, keine Replikation. Keine Netzwerk-Bibliothek im Produktionsbaum (`deny.toml`, `cargo tree -e normal`).
- **Test:** T-ARCH-4

## VETO-FAILCLOSED: Keine stillen Fallbacks

- **Quelle:** P2, P6, FM-10, FM-16
- **Status:** gültig
- **Regel:** Unbekannter Zustand, korrupte Daten, abgelaufene Fristen führen zu Verweigerung oder `Pending`. Kein `unwrap_or_default` auf I/O, kein `let _ =` auf `Result`, kein `unwrap_or(u64::MAX)` als Sentinel. Ein Fehler ist sichtbar.
- **Test:** T-LINT-2, T-FSYNC-1

## VETO-D01b: Schlüsselvernichtung ist Teil von v1

- **Quelle:** D-01, KF-1, SPEC 3.3
- **Status:** gültig. Ersetzt VETO-D01 (Fassung 2.0.0, „keine Ebene B in v1“), das aufgehoben ist.
- **Regel:** Löschen in v1 umfasst drei Säulen: Gate und Purge, Schlüsselvernichtung über den Tresor und unabhängige Messung. Die Reichweite der Schlüsselvernichtung wird je Schicht ausgewiesen (`key_coverage`, D-69). Die rechtliche Wertung bleibt beim Betreiber (E-L2).
- **Test:** T-VAULT-1 bis T-VAULT-5, T-KEYCOV-1

## VETO-F02: Keine regionale HNSW-Neuverdrahtung

- **Quelle:** D-77 (ersetzt die Formulierung der Fassung 2.0.0)
- **Status:** gültig, präzisiert
- **Regel:** Verboten sind partielles Rewiring und lokales Re-Graphing einzelner Regionen. Erlaubt sind genau: (1) Vektor-Surrogat vor `Gated`, (2) eine **lineare Batch-Kantenreparatur** über alle Adjazenzlisten im Purge-Fenster, (3) ein **vollständiger deterministischer Rebuild** nach Quote. Slot-Nummern werden erst nach Reparatur in die Freiliste aufgenommen.
- **Test:** T-VEC-DEL-1, T-VEC-DEL-2, T-VEC-REBUILD-INDEP

## VETO-F10: Keine mandantenlose Zugriffsfunktion

- **Quelle:** D-07, D-61, T-API-1, T-ISO-1
- **Status:** gültig
- **Regel:** Jede Funktion benötigt einen `TenantContext`. `TenantPolicy` entfällt. Kein mandantenübergreifender Datenfluss, kein Cross-Tenant-Index. Zugriff auf den Rohspeicher nur über `TenantScopedStorage` (AR-6).
- **Test:** T-API-1 (trybuild), T-TENANT-1

## VETO-D05: Keine Schalter für Verschlüsselung und Integrität

- **Quelle:** P5, D-05, HR-13
- **Status:** gültig
- **Regel:** Kein unverschlüsselter Modus, kein `Option<KeyManager>`, kein Feature für Integritäts-Downgrade, kein Schlüssel für ältere WAL-Formate. Ausnahmen gelten nur für `process_hardening` (D-63) und das Feature `testing` (D-46), das nur den Argon2-Schritt ersetzt und nie im Release aktiv ist.
- **Test:** T-DEF-1, T-CFG-1, T-CFG-2

## VETO-D06: Passphrase nur über Argon2id

- **Quelle:** D-06, D-68
- **Status:** gültig
- **Regel:** Die Passphrase wird nur mit Argon2id abgeleitet, innerhalb von `m` ∈ [64 MiB, 4 GiB], `t` ∈ [3, 16], `p` ∈ [1, 16]. Die Grenzen werden vor der Ableitung geprüft. HKDF dient ausschließlich der Subschlüssel-Ableitung aus `K_master`.
- **Test:** T-KDF-1, T-KDF-2

## VETO-D27: Keine automatische Entitäts- oder Metadatenextraktion

- **Quelle:** D-27, SPEC 1.4
- **Status:** gültig
- **Regel:** `AutoExtractionMode` und jede LLM-basierte Extraktion entfallen. Der Aufrufer liefert Metadaten.
- **Test:** T-ARCH-3

## VETO-D29: `unsafe` nur in `sys` und `simd`

- **Quelle:** D-29, AR-8
- **Status:** gültig
- **Regel:** `unsafe` ist ausschließlich in `contextra-sys` und `contextra-simd` erlaubt, mit `// SAFETY:`-Kommentar. Alle anderen Produktcrates setzen `#![forbid(unsafe_code)]`. Eine `unsafe fn` darf von keinem Crate mit `forbid` aufgerufen werden; die Funktion wird dann nicht angeboten.
- **Test:** T-UNSAFE-1

## VETO-D94: Keine Speicherabbildung (`mmap`)

- **Quelle:** D-94, D-29
- **Status:** gültig
- **Regel:** Kein `mmap` und kein `memmap2`. Lesezugriffe auf WAL, SSTable, Textsegment und Snapshot laufen über positionelles Lesen des Ports `Vfs`. Begründung: Klartextseiten im Page-Cache entziehen sich der Kontrolle des Löschkerns.
- **Test:** T-UNSAFE-1, T-LINT-2

## VETO-D03: Kein `block_on` im Bibliothekscode

- **Quelle:** D-03, F-14
- **Status:** gültig
- **Regel:** Synchrone Ports über asynchrone I/O und `block_on` sind verboten. Der `WasmMergeOperator` mit `block_on` entfällt (D-84).
- **Test:** T-LINT-2

## VETO-D04: Ein `Vfs`, keine direkten Dateizugriffe

- **Quelle:** D-04, F-13, F-29
- **Status:** gültig
- **Regel:** Ein einziger Trait `Vfs` in `ports`, Implementierung `RealVfs` in `durable-fs`. `store`, `vector`, `text` und `erasure` nutzen nur `Vfs`.
- **Test:** T-LINT-2

## VETO-D55: Kein Wiederholen nach `fsync`-Fehler

- **Quelle:** D-55, I-10, FM-5
- **Status:** gültig
- **Regel:** Ein Fehler von `sync_data`, `sync_all` oder `sync_dir` macht die betroffene Datei fatal. Derselbe Handle wird nie erneut synchronisiert. Der Store wechselt in `Poisoned`.
- **Test:** T-FSYNC-1

## VETO-D73: Der WAL wird nie umgeschrieben

- **Quelle:** D-73, I-3
- **Status:** gültig
- **Regel:** Zielframes werden nicht aus WAL-Segmenten herausgeschnitten. Entfernt werden ganze Segmente bis zum Flush-Punkt. Die HMAC-Kette wird über `WalFloor` fortgesetzt.
- **Test:** T-WAL-2, T-ISO-1

## VETO-D57: Keine Dokumentlocks für Subject, Collection und Tenant

- **Quelle:** D-57, D-78
- **Status:** gültig
- **Regel:** Dokumentlocks gelten nur für `EraseTarget::Document`. Für Subject, Collection und Tenant gibt es kein Lock-Set. Die Gate-Barriere ist O(1).
- **Test:** T-GATEGROUP-1, T-CONC-1

## VETO-D75: Keine Klartext-Namen in Pfaden

- **Quelle:** D-75, D-09, HR-4
- **Status:** gültig
- **Regel:** Datei- und Verzeichnisnamen enthalten nur Tags, `file_id`, `erasure_id` oder feste Namen. Keine Mandanten- oder Collection-Namen im Dateisystem.
- **Test:** T-RAW-6

## VETO-D77: Keine Nullvektoren als Löschung

- **Quelle:** D-77, D-18
- **Status:** gültig
- **Regel:** Ein Vektor wird nicht durch Nullen ersetzt, das die Navigation zerstört. Löschung erfolgt über Surrogat (Mittelwert der Nachbarn bzw. Einstiegsknoten), Batch-Kantenreparatur und Rebuild.
- **Test:** T-VEC-DEL-1

## VETO-HR3: Kein Schlüssel-Escrow

- **Quelle:** HR-3, D-43
- **Status:** gültig
- **Regel:** Der Hersteller hält keinen Schlüssel und kann keine Daten wiederherstellen. Zusätzliche Keyslots des Betreibers sind Betreiberentscheidung.
- **Test:** Doku-Test, Review

## VETO-D15: Aufbewahrungsattestation ist Pflicht

- **Quelle:** D-15, E-L7
- **Status:** gültig
- **Regel:** Jeder Löschauftrag trägt eine `RetentionAttestation`. Contextra prüft die Erklärung nicht inhaltlich, führt sie aber signiert im Bericht.
- **Test:** T-API-2

## VETO-D51: `soft_delete` ist keine Löschung

- **Quelle:** D-51, KF-6
- **Status:** gültig
- **Regel:** Es gibt kein `delete`. Die Dokumentation von `soft_delete` beginnt mit „Für Löschpflichten `erase` verwenden.“
- **Test:** T-API-3, T-LINT-1
