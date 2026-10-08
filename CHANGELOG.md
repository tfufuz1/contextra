# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Version 0.1.0 ist derzeit unveröffentlicht.

## [Unreleased]

## [0.1.0] - unveröffentlicht

### Fixed
- RevocationLog-Härtung durch Persist-before-Mutate, signierte Marker und inkrementelle Verifikation (`a8e512d6`, #4440)
- Vollständiger Konsum aller Versionen desselben Schlüssels beim LSM N-Way Compacting (`1b6db180`, #4439)
- HNSW-Index-commit() Korrektur mit Op-Normalisierung, Fehleratomarität und RAM-Vektor-Nullung bei Slot-Freigabe (`3a4c5192`, #4438)
- Direkte SIMD-Kernel-Tests und Basisverifikation für Sicherheitsinvariante in Fallback-Pfaden (`eaa0df92`, #4436)

### Security
- Verhinderung von Script-Injection in CI-Workflows und Einschränkung von Workflow-Berechtigungen (`79519282`, #4437)
