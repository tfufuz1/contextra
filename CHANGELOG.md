# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Version 0.1.0 ist derzeit unveröffentlicht.

## [Unreleased]

### Removed
- Entkopplung und Entfernung von 16 Parkplatz-Crates aus dem Workspace gemäß ADR-PARK-001 (gesichert unter Git-Tags `park/EP1/v1` bis `park/EP6/v1`):
  - `contextra-adapt`
  - `contextra-agent`
  - `contextra-audit-export`
  - `contextra-avv-generator`
  - `contextra-cognition`
  - `contextra-core`
  - `contextra-graph`
  - `contextra-infer-candle`
  - `contextra-infer-ollama`
  - `contextra-infer-onnx`
  - `contextra-kvcache`
  - `contextra-license`
  - `contextra-privacy`
  - `contextra-py`
  - `contextra-router`
  - `contextra-sandbox`
- Entfernung nicht-essentieller und experimenteller Cargo-Features gemäß Abschnitt 3.4 (Aktion E):
  - `contextra-types`: `auto-extraction-opt-out`
  - `contextra-crypto`: `cloud-egress-guard`
  - `contextra-rank`: `dibud`
  - `contextra-store`: `deletion-proof`
  - `contextra-vector`: `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`
  - `contextra-engine`: `default`, `graph`, `adapt`, `kvcache`, `candle`, `sandbox`, `onnx`, `reranking`, `graph-connectivity-health`, `edge-reinforcement-learning`, `entity-extraction`, `auto-extraction-opt-out`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `experimental-diskann`
  - `contextra-db`: `sandbox`, `reranking`, `onnx`, `experimental-diskann`, `graph-connectivity-health`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `volatile-vault`, `edge-reinforcement-learning`
  - `contextra`: `router`, `ollama`, `candle`, `onnx`, `privacy`, `sandbox`, `adapt`, `cognition`, `agent`, `graph`, `kvcache`, `license`, `sovereign`

## [0.1.0] - unveröffentlicht

### Fixed
- RevocationLog-Härtung durch Persist-before-Mutate, signierte Marker und inkrementelle Verifikation (`a8e512d6`, #4440)
- Vollständiger Konsum aller Versionen desselben Schlüssels beim LSM N-Way Compacting (`1b6db180`, #4439)
- HNSW-Index-commit() Korrektur mit Op-Normalisierung, Fehleratomarität und RAM-Vektor-Nullung bei Slot-Freigabe (`3a4c5192`, #4438)
- Direkte SIMD-Kernel-Tests und Basisverifikation für Sicherheitsinvariante in Fallback-Pfaden (`eaa0df92`, #4436)

### Security
- Verhinderung von Script-Injection in CI-Workflows und Einschränkung von Workflow-Berechtigungen (`79519282`, #4437)
