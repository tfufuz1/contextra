# AGENTS.md — `contextra-kvcache`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-kvcache` implementiert Prefix-Radix-Bäume, KV-Blöcke, mandantenisoliertes Caching, Tiering und AEAD-Segmentverschlüsselung für modernes KV-Cache-Management (Ring 1).
Als eigenständiger Baustein beschleunigt er LLM-Inferenz und Context-Prefilling durch deterministisches Teilen und Auslagern von Key-Value-Zuständen.
Gemäß den Entkopplungs-Richtlinien ist `contextra-kvcache` vollständig von jeglicher Lösch- und DSGVO-Logik getrennt; Speicherbereinigung erfolgt als neutrale Invalidierung/Verdrängung.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 1 (State & Storage, KV-Block Cache)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-crypto` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Mandanten-Isolation:** Strikte logische und kryptografische Trennung von Cache-Einträgen nach `TenantId`.
- **Zero-Panic-Doktrin:** Vollständiger Verzicht auf `unwrap()` und `expect()` im Produktionscode.
- **Entkopplung (Decouple-Inspector):** Völlige Abwesenheit von Löschketten-, Saga- oder Rechtsbegriffen (maschinell geprüft via `tools/decouple_inspector.py`).

## 4. Verbotene Muster
- Kein `unsafe`-Code.
- Keine unverschlüsselte Persistenz oder unverschlüsseltes Tiering sensibler Prompt-Tokens.
- Kein Cross-Tenant Cache-Sharing ohne kryptografische Barriere.
- Keine direkten Abhängigkeiten zu konkreten Inferenz-Frameworks im Kern des Caches.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Konsumiert:** `Clock`-Port, Verschlüsselungsprimitiven aus `contextra-crypto`.
- **Bereitgestellt für:** `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-engine`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Mandanten-Isolations-Tests:** Verifikation, dass Tenant A niemals auf Cache-Blöcke von Tenant B zugreifen kann.
- **Decouple-Inspector Audit:** Automatisierter Leak-Check auf unerwünschte Domänen- oder DSGVO-Begriffe.
- **Fuzzing & Sabotage:** Cache-Verdrängungs- und Invalidierungs-Tests unter künstlichem Speicherdruck.

## 7. Offene Punkte & Technische Schulden
- Keine.
