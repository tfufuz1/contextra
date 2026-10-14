# Internes Briefing: Was Contextra bereits verwendet, und was der Prompt bewusst nicht enthält

Dieses Dokument ist nur für dich. Es darf nicht in den Gemini-Prompt.

## 1. Quellenlage und Vorbehalte

- Der Ist-Stand stammt aus den fünf hochgeladenen Berichten. Ich habe ihn in dieser Runde nicht vollständig am Code nachgeprüft.
- Die Berichte widersprechen sich teilweise. Beispiele:
  - Ein Bericht beschreibt den Forward-Push-PPR als einzigen Graphkern. Der Gegenbericht vom 26.09. nennt zusätzlich einen `TlHfd`-ShadowMode.
  - Ein Bericht schreibt, die Konsolidierung sei ein Fernziel. Der Gegenbericht zeigt eine zweistufige Pipeline in Produktionscode.
  - Die Berichte beziehen sich auf unterschiedliche Commits (`bd71f801`, `4b9387d6`, `b28d5186`). Dein aktueller HEAD ist `16764583`.
- Die Audit-Befunde vom 13.10. habe ich dagegen am aktuellen Code geprüft (siehe `Audit_Befunde_2026-10-14.md`).

## 2. Bereits verwendete oder implementierte Verfahren (laut Berichten)

| Feld | Verfahren | Ort laut Bericht | Einschätzung |
|---|---|---|---|
| Graph | Forward-Push-PPR (Andersen-Chung-Lang) mit Stern-Expansion für Hyperkanten | `contextra-graph` | bestätigt in mehreren Berichten |
| Graph | TL-HFD (Thresholded Local Hyper-Flow Diffusion) im ShadowMode | `contextra-graph/src/tl_hfd/` | laut zwei Berichten implementiert, nicht unabhängig verifiziert |
| Graph | Bidirektionaler Dijkstra für Pfade | `path_rag` | bestätigt |
| Graph | Leiden-Community-Detection | `contextra-graph` | laut Bericht vorhanden |
| Fusion | RRF, gewichtete RRF, Resonance-Bonus, Provenance-Tracking | `contextra-rank/fusion/` | bestätigt |
| Fusion | DiBud (budgetierte RRF) mit Feature-Flag | `contextra-rank/dibud/` | laut Bericht vorhanden |
| Kalibrierung | Isotonic (PAVA), Platt | `contextra-rank/calibration/` | bestätigt |
| Kalibrierung | Conformal-Calibrator | `contextra-rank` | laut einem Bericht vorhanden, in einem anderen als Lücke geführt. Bitte prüfen. |
| Bandits | LinUCB mit Sherman-Morrison und Diagonal-Approximation | `contextra-adapt/bandit.rs` | bestätigt |
| Bandits | Flow-Corrected Thompson Sampling | `contextra-adapt` (Feature), `contextra-router/fc_ts_dispatch.rs` | laut Bericht vorhanden |
| Bandits | Lyapunov-Drift-Watcher, DriftPolicyBridge, PID-Latenzregler | `contextra-adapt` | bestätigt |
| Index | HNSW mit Arena-Allokator, RaBitQ, SQ8, DiskANN-Modul | `contextra-vector` | laut Berichten vorhanden |
| Index | Filtered Search (ACORN-Schnittstelle), Graph-Reparatur bei Löschung | `contextra-vector` | laut Bericht vorhanden |
| Speicher | Tier-basierte Compaction, WAL mit HMAC-Verkettung, Group-Commit | `contextra-store` | teilweise bestätigt |
| Transaktionen | MVCC, SSI-Validator (`SequenceLogSsiValidator`) | `contextra-mvcc` | bestätigt (Commit-Pfad ruft ihn auf) |
| Kryptografie | Ed25519-Löschbeweise, Crypto-Shredding, AEAD-Tier-2 | `contextra-crypto`, `contextra-kvcache` | bestätigt |
| Kognition | Structural Pass (deterministisch), Generative Synthesis Pass mit Kostenschutz, Community-Stabilitäts-Tracker | `contextra-cognition` | laut Bericht vorhanden |
| Kognition | LeanRAG-Aggregation | `contextra-cognition/aggregation_phase.rs` | laut Bericht vorhanden. Der Bericht vom 26.09. nennt sie „zu integrieren“. Widerspruch. |
| Sicherheit | DLP/Egress-Klassifikator, Prompt-Injection-Schutz, GASP-Grounding mit Isotonic | `contextra-privacy`, `contextra-mcp`, `contextra-infer-candle` | laut Bericht vorhanden |
| KV-Cache | Prefix-Radix-Baum, Tier-2-Spill, Tenant-Isolation, KvReusePolicy (nur längenbasiert) | `contextra-kvcache` | bestätigt, keine inhalts- oder attention-basierte Eviction |
| SIMD | AVX2/AVX-512/NEON-Kernel, u8-Distanzen, Dispatch | `contextra-simd` | bestätigt |

## 3. Bekannte Lücken laut Berichten

- Automatische Entity-/Relation-Extraktion aus Text fehlt. Im README als „Not Implemented“ markiert.
- Kein inhalts- oder attention-basiertes KV-Cache-Eviction und keine Quantisierung der KV-Werte.
- Kein Pfad-Budget für mehrere disjunkte Pfade (`k-path`).
- Konforme Garantie für die Fusion fehlt (laut einem Bericht).
- Budgetprüfung vor LeanRAG-Kompaktierung (`estimate_compaction_peak_bytes`) fehlt.
- `child_edge_ids` ist im Schema und im Struct vorhanden, aber `cascade.rs` durchläuft es nicht rekursiv.
- SSI-Read-Set-Tracking ist laut einem Bericht nicht in der Lesepfad-Schnittstelle verdrahtet. Der Audit vom 13.10. zeigt aber, dass der Commit-Pfad SSI aufruft. Widerspruch, bitte prüfen.

## 4. Warum der Prompt so aufgebaut ist

- **Kein Projektname, keine Invariantennummern (P24, P28 …), keine Crate-Namen.** Gemini hat keinen Git-Zugriff. Hinweise auf den Code würden die Recherche auf bekannte Lösungen verengen. Stattdessen beschreibt das Profil (Abschnitt 1) die Eigenschaften in allgemeinen Begriffen.
- **Schwerpunkt außerhalb KV-Cache und Token-Kompression.** Das entspricht deiner letzten Vorgabe. Die ursprünglichen Beispiele (Ultra-Low-Bit, Token-Kompression) bleiben als Nebenfeld M erhalten.
- **Die Berichte enthalten viele Algorithmen, die bereits umgesetzt sind.** Der Prompt sagt Gemini nicht, was schon da ist. Das würde verzerren. Der Prompt enthält eine Seed-Liste, die ausdrücklich als unverifiziert gekennzeichnet ist.
- **Seed-IDs sind teils fehlerhaft.** Einige Platzhalter-Muster (`2405.12345`, `2006.12345`, `2404.12345`, `2406.12345`) stammen aus den Vorberichten und sind keine echten Papers. Der Prompt schließt sie aus. Bei SnapKV und KIVI habe ich Zweifel an den genannten IDs. Nach meinem Kenntnisstand wären KIVI `2402.02750` und SnapKV `2404.14469`. Bitte prüfen, bevor du dich darauf stützt.
- **Determinismus, Lokalität und Zero-Panic stehen als harte Randbedingungen im Prompt**, weil sie die Auswahlkriterien sind. Ohne sie empfiehlt Gemini u. U. Verfahren, die für dich unbrauchbar sind.
- **Das Ausgabeformat zwingt zu mikrofeingranularen Spezifikationen** (Pseudocode, Komplexität, Determinismus, Fehlerbehandlung, Rust-Hinweise). Das entspricht deinem Ziel.
- **Gewichtung in der Bewertungsmatrix:** Korrektheit und Determinismus vor reiner Geschwindigkeit. Das folgt aus dem Systemprofil.

## 5. Vor dem Einsatz des Prompts

1. Gemini-Deep-Research hat ein Tageslimit. Bitte den Prompt einmal vollständig einreichen, nicht in Teilen.
2. Nach dem Ergebnis: Jede empfohlene ID gegen arXiv prüfen. Die Seed-Liste verwenden, um zu sehen, ob Gemini sie gefunden und korrekt zitiert hat.
3. Die Ist-Stand-Widersprüche aus Abschnitt 1 vor einer Umsetzung am Code klären (`grep` nach `tl_hfd`, `conformal`, `aggregation_phase`, `estimate_compaction_peak_bytes`, `child_edge_ids`).
