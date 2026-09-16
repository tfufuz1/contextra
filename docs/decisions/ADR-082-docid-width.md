# ADR-082: DocId-Breite — 128-Bit BLAKE3-Truncation (Final)

* **Status:** Final
* **Datum:** 2026-09-16
* **Kontext / Auslöser:**
  Gemäß dem unabhängigen Review-Befund in `MEMFUSE_UNABHAENGIGE_REVIEW_2026-09-16.md`, Abschnitt 3.7, erzeugte die ursprüngliche `DocId::from_key` eine 64-Bit-Dokumenten-ID, indem der BLAKE3-Hashwert eines Zeichenketten-Schlüssels (`key`) auf seine ersten 8 Bytes (Little-Endian `u64`) trunkiert wurde.

  Das System behandelte Kollisionen in `memfuse-db::Collection` fail-safe über Reverse-Lookups. Für Version v0.x gilt die normative Kapazitätsgrenze von **100 Millionen Dokumenten pro Collection** (Kollisionswahrscheinlichkeit $p(k) \approx 0{,}27\%$).

  Zur Vorbereitung der Enterprise-Milliarden-Skalierung führt ADR-082 die kanonische 128-Bit `DocId` ein.

## Finale Entscheidung
Es wird die **128-Bit BLAKE3-Truncation (16 Bytes)** als kanonischer Zieltyp verabschiedet.

**Ausschluss von UUIDv7:**
UUIDv7 wird **ausdrücklich ausgeschlossen**. UUIDv7 ist zeitbasiert/zufallsbehaftet und nicht-deterministisch. Die Verwendung von UUIDv7 würde den ADR-016-Fail-Safe-Mechanismus zerstören und eine zusätzliche persistente Schlüssel→ID-Zuordnungstabelle erfordern, was eine erhebliche Architektur-Regression darstellen würde.

**BLAKE3-128 Präzisierung:**
`DocId::from_key(key)` bleibt eine rein deterministische, zustandslose Funktion des Original-Schlüssels. Bei 128 Bit ($N = 2^{128} \approx 3{,}4 \times 10^{38}$) sinkt die Kollisionswahrscheinlichkeit bei $10^{12}$ (1 Billion) Dokumenten auf $p(k) \approx 10^{-15}$.

## Schema-Versionierung & Migrationswerkzeug
1. **Cargo Feature-Flag:** Das `docid-128` Feature-Flag in `memfuse-core` erlaubt die stufenweise Aktivierung.
2. **Schema-Version 2:** Manifest & SSTable Version 2 kennzeichnet das 128-Bit-Format.
3. **Migrations-Tool:** `cargo xtask migrate-docid-128` führt die deterministische Re-Derivierung von 64-Bit V1-Exports in 128-Bit V2-Exports durch.

## Konsequenzen
* `GESAMTSPEZIFIKATION.md` und `ARCHITECTURE.md` dokumentieren normativ die v0.x 100-Mio.-Dokumenten-Grenze sowie das 128-Bit V2-Schema.
* Downstream-Crates (`memfuse-store`, `-index`, `-graph`, `-db`, `-mcp`, `-py`) werden in separaten Folge-Prompts auf `docid-128` umgestellt.
