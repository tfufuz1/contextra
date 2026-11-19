# contextra — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §14 und `docs/crates/contextra.md`. **Ausgangslage:** Die Fassade enthält Agent-Memory, Boot, Profile und Query-Rewriter, eine Default-Feature-Aktivierung sowie mehr API als der Sollzustand zulässt.

1. Reduziere die Fassade auf dünne Delegation an DB/Erasure: `open`, `collection`, `insert_k`, `insert_p`, `hybrid_search`, `erase_subject`, `drop_collection` und konfigurierte Compliance-Profile.
2. Mache `ContextraConfig` aus Types kanonisch und biete `enterprise_regulated(data_dir, passphrase)` ohne versteckte Fallback-Schlüssel oder optionale Erasure-Konfiguration.
3. Entferne Agent-Memory, Query-Rewrite, Boot/Lizenz-/Performance-Altlogik oder migriere sie in nicht mehr vorhandene/extern zu liefernde Endpunkte; die Fassade besitzt keine Speicher- oder Indexlogik.
4. Setze `default = []`, beschränke Dependencies auf `contextra-db`, `contextra-types`, `contextra-erasure` und beseitige den Produktions-Panikpfad im Builder.
5. Schreibe API-Integrationstests für K/P-Insert, Suche, Subject-Erasure und Collection-Drop; Weitergabe von `Erased`/`Restricted` darf nicht nivelliert werden.

**Abnahme:** Public-API-Snapshot entspricht Spezifikation, Dependency-Gate, Konfigurations-Negativtests und End-to-End-Erasure-Smoketest.
