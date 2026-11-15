# Bewusst nicht extrahierte Altquellen

Die Spezifikation (docs/SPEC.md) bestimmt, was uebernommen wird. Folgende Altquell-Crates haben **keine** Entsprechung
oder widersprechen ausdruecklich einer Festlegung. Sie liegen nicht im Archiv.

| Altquelle (Crate) | Grund |
|---|---|
| `graph` (23,6k LOC) | Graphindex und PPR kommen in der Spezifikation nicht vor (Prinzip 1: Anforderung vor Mechanismus) |
| `kvcache` | KV-/Prefix-Cache gehoert in den separaten Inferenzdienst (11.2); persistente Prompt-Caches sind fuer P verboten |
| `cognition`, `agent` | autonome Agenten/Memory-Konsolidierung sind Nicht-Ziel (1.4) |
| `router`, `adapt` | Bandit-/Kaskadenrouting ohne Anforderung; Inferenzwahl ist Betreiberentscheidung (11.3) |
| `infer-candle`, `infer-onnx` | Inferenz im Kernprozess ist verboten (11); Modelle laufen als separater Dienst |
| `mcp`, `py` | andere Schnittstellen als HTTP/JSON (18.5) |
| `store`: memtable, compaction, lsm-Engine, system_pressure, block_cache | kein LSM-Baum (5.1); append-only Segmentspeicher |
| `mvcc`: ssi, tx_buffer | ein Schreiber je Mandant, Leser gegen Schnappschuesse (3.6); keine Mehrversions-Transaktionen |
| `vector`: Dateiformate der Altindizes | Plattenformat nach 5.1/Umschlag v1 ist Neubau; HNSW/DiskANN siehe `_reserve/ann-backend` |
| `types`, `ports` (Hubs) | HR-1..HR-3 verbieten Sammel-Crates; nur Einzelteile wurden in die jeweiligen Ziel-Crates uebernommen |
| `db`, `engine`, Fassade | alte Integrationsschicht; Anwendungsfaelle entstehen neu in `acta-engine` (Kap. 10, 19) |
| `benchmarks`, `reports`, `archive` (Altlast), Audit-/Kampagnen-Tests (`campaign_*`) | Berichte und Kampagnenartefakte der Altquelle, nicht uebertragbar |
| Windows-Pfade (`acl_win32`) | Plattform ist Linux x86_64/aarch64 (1.2) |

Luecken ohne Altquelle (**Neubau**): `acta-access` (ABAC), `acta-pstore`, Git-Objektmodell in `acta-history`,
`acta-decision`, Fachfunktionen FF1-FF9, `acta-api` (axum/OIDC/mTLS), `acta-sink-net`, `acta-cli`, `acta-server`,
Landlock/seccomp-Sandbox, Festkomma-`fx_ln`, DE/EN-Stemmer nach 9.2.
