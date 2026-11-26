# IMPL-08 — Security, Gates G1–G7, Capabilities, API/MCP, Server/CLI

**Phase:** P4–P5 · **SOLL:** SPEC §4.5, §8, §9 · **Abhängig von:** IMPL-05, IMPL-07 · **Crates:** neu `contextra-security` (L4), `contextra-agent`, `contextra-mcp`, neu `contextra-server`, neu `contextra-cli` (L6)

## 1. Ziel

Sicherheitsfunktionen als eigene Crate: Schlüsselhierarchie, Handles, Trust/Taint, Capabilities, Merge-Gates. Außenschicht (MCP/HTTP/CLI) ruft ausschließlich die Kernel-API.

## 2. Kryptografie (`contextra-security`)

- **Schlüssel:** Passphrase → Argon2id (`crypto::kdf`, übernommen) → **KEK**. Pro Pack/Segment eine **DEK** (zufällig, 256 bit), mit KEK gewrappt im Pack-/Segment-Header. KEK-Rotation = nur DEKs neu wrappen (O(Anzahl Packs), kein Datenrewrite).
- **AEAD:** XChaCha20-Poly1305 (Default). Nonce-Schema (pack_id, extent_no, counter) – nie wiederverwendet (Property-Test).
- **Content-Hash:** `BLAKE3::keyed_hash(hkdf(repo_seed, "contextra-content-v1"), plaintext)` (INV-5) — verhindert Hash-Probing durch Dritte.
- **Handle-MAC:** wie IMPL-01 (`contextra-handle-mac-v1`, 10 bit). Ungültiges MAC ⇒ `Error::InvalidHandle` ohne Unterscheidung „existiert nicht".
- **Commit-Signatur:** Ed25519 (`ed25519-dalek` zieht aus `crypto` hierher um; IMPL-02 A6), optional pro Commit; Schlüssel in Keyring-Port.
- Speicherhygiene: `zeroize` für Schlüsselmaterial; `anti_tamper::VolatileEncryptionKey` übernehmen.
- **Nicht** enthalten: Crypto-Shredding, Löschnachweise (C-15).

## 3. Trust und Taint

- `TrustTier` (0..4, Meet = `min`), `TaintFlags` (Join = Vereinigung). Abgeleitete Zelle: `trust = min(inputs)`, `taint = ∪ inputs`.
- Promotion nur über explizite, protokollierte Operation (`promote(cell, to_tier, evidence)`), erfordert Capability `trust.promote`; Eintrag im `provenance`-Tree.
- `INJECTION_SUSPECT`: Muster-Detektor (Regeln als Daten, versioniert) beim Ingest; Zelle bleibt speicherbar, wird bei `recall` standardmäßig **nicht** in Anweisungskontext gegeben (Flag im `ContextPack`, Erzwingung in Gate G5 und API-Default).

## 4. Capabilities

Token-basiert (`Capability { actor, ns_scope, ops: BitSet, expires_hlc, branch_scope }`), signiert/MAC'd mit Repo-Schlüssel; Prüfung in **jedem** Kernel-Einstieg (nicht nur im MCP-Layer). Ops: `read, write, retire, merge, promote, admin, purge`. Standard-Agentenrolle: `read + write(session-branch)`; Merge nach `main` nur mit `merge` und bestandenen Gates.

## 5. Merge-Gates G1–G7

Laufen in dieser Reihenfolge vor Publish eines Merge-Commits; jedes Gate liefert `Pass | Fail{reason} | NeedsReview`.

| Gate | Prüfung (Kurzfassung; normativ SPEC §4.5) |
|---|---|
| G1 | Strukturintegrität: Prolly-Root-Hashes konsistent, Commit-DAG azyklisch |
| G2 | Schema/Policy: Namespace-Policies (`policies`-Tree) eingehalten |
| G3 | Capability: Aktor darf in Ziel-Scope schreiben/mergen |
| G4 | Konflikte: keine ungelösten Einträge im `conflicts`-Tree |
| G5 | Taint: Zellen mit `INJECTION_SUSPECT`/`UNVERIFIED_TOOL` nicht in höheren Trust-Tier ohne Review |
| G6 | Trust: `trust(neu) ≤ trust(Quelle)`; keine stille Promotion |
| G7 | Quoten/Limits: Größe, Anzahl Zellen, Rate je Aktor |

Die genaue Semantik je Gate MUSS beim Implementieren gegen SPEC §4.5 abgeglichen werden (`OPEN-08-1`: Tabelle ist Zusammenfassung des Autors, der Wortlaut der Quelle gilt). Gates sind reine Funktionen `fn gate(&MergeContext) -> GateResult`, einzeln testbar.

## 6. API (`contextra-agent`, MCP)
