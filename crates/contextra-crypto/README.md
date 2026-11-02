Kryptoprimitive: AEAD, Argon2id, HKDF, HMAC, Ed25519, Tags.

## Status

Ring 1. Ist-Zustand: vorhanden; Untergrenze der KDF zu niedrig und ohne Obergrenze (F-11); Feature-Altlasten (F-21, F-23); Tests und Benchmarks kompilieren nicht vollständig (3.0.0). Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
