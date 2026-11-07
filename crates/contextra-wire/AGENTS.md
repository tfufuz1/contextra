# AGENTS.md — `contextra-wire`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-wire` stellt ein autarkes, zero-copy Wire-Protokoll und Serialisierungs-Subsystem (Ring 0) auf Basis von FlatBuffers und gehärtetem JSON-RPC 2.0 bereit.
Es dient als standardisierte, versionierte Datenaustausch-Schicht zwischen Prozessen, IPC-Grenzen und externen Clients (z. B. Python, MCP, Netzwerkgateways).
Als eigenständiger Baustein kapselt es Wire-Formate vollständig, ohne Domänen- oder Speicherdurchführungslogik zu enthalten.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Wire-Protokolle & Serialisierung)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Kapselung von generiertem Code:** Auto-generierter FlatBuffers-Code verbleibt strikt isoliert in `src/generated.rs` mit `@generated`-Markierung.
- **Handgeschriebener Code-Standard:** `src/lib.rs` erzwingt streng `#![deny(unsafe_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- **Gehärtete Validierung:** Verifizierter Root-Zugriff (`root_with_opts`) ohne ungeschützte `root_unchecked`-Aufrufe.
- **DoS-Schutz & Eingabebeschränkungen:** Strikte Durchsetzung von Größen-, Tiefen- und Batch-Limits sowie Erkennung doppelter Schlüssel via `BTreeSet`.
- **Zero-Panic-Doktrin:** Parsing- und Deserialisierungsfehler resultieren deterministisch in standardisierten Fehlercodes (-32700 bis -32603).

## 4. Verbotene Muster
- `unsafe`-Code außerhalb streng abgesicherter generierter FlatBuffers-Bindings.
- Direkte I/O-, Netzwerk- oder Persistenzaufrufe im Crate.
- Leaks von internen Speicherkerntypen in Wire-Schemata.
- Unbegrenzte Pufferallokation bei untrusted Payloads.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- Stellt Serialisierer, Deserialisierer und Verifizierer für `contextra-py`, `contextra-mcp` und Inter-Crate-IPC bereit.

## 6. Tests, Sabotage & Qualitätssicherung
- **Schema-Drift-Prüfung:** `check-flatbuffers-drift` zur Validierung der Konsistenz zwischen Schemadefinitionen und generiertem Code.
- **Maligne Payload Fuzzing:** Fuzzing gegen Pufferüberläufe, Endlosschleifen und Deep Recursion Denial-of-Service Angriffe.
- **Mutations-Testing:** Prüfung der Fehlerbehandlung bei absichtlich korrumpierten Wire-Frames.

## 7. Offene Punkte & Technische Schulden
- Keine.
