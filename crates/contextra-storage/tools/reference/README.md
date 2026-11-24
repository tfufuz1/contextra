# Unabhängiger R8-Referenzcodierer

Besitzer: SP-VERIFY. Dieses Werkzeug erzeugt und prüft kanonische R8-Bytes unabhängig vom Rust-Produktionscodec. Der Codierer darf keine Rust-Implementierung aufrufen, keine Produktionsdateien ändern und keine Schlüssel oder Nutzdaten aus realen Stores verwenden. Vor G-F ist ein zweiter unabhängiger Leser/Codierer erforderlich; dieses Verzeichnis allein ist noch keine Formatfreigabe.

## Eingabe und Ausgabe

- Eingabefälle sind versionierte, textuelle Fixtures mit ausschließlich synthetischen IDs und deterministischen Testschlüsseln.
- Ausgabe pro Fixture: kanonische Bytes in Hex/Binärform, strukturierte erwartete Felder, Formatversion, SHA-256 des Vektors und Decoder-Urteil.
- Nicht-kanonische Varianten werden als negative Fixtures geführt: abgeschnitten, überlang, falsche Reihenfolge, unbekanntes Tag/Feature, reservierte Bits, falsche Prüfsumme/MAC und Längenüberlauf.
- Golden-Dateien werden niemals automatisch durch den zu testenden Codec aktualisiert. Änderungen brauchen Review, Spec-Verweis, Versionsentscheidung und Migrationsauswirkung.

## Pflichtvektoren vor G-F

Dateikopf/FORMAT/KEYS; Redo-Gruppe und Change/Gate/Control/Reserve-Frames; Catalog-Snapshot/Edit-Kette mit `reserve_space_ids`; Space-ID-Werte `0`, `255`, `256`, `u32::MAX`, `u64::MAX` und `Exhausted`; ClusterSection/Directory/PackMeta/SubIndex/DataBlock/RangeDel/Bloom/Footer; Blob-Record und Repack mit stabiler `record_uid`; Ledger-Kette mit Domain-separierten Cluster-/Space-Tomb-HMACs, Scope-Digest, Request-ID/Digest und allen Receipt-Statuswerten; Branch-Control-Frames; Extension-Manifest/Attestation; Rohkopie und negative Cross-Cluster-Replay-Vektoren.

Die HMAC-Vektoren kodieren die exakten Spec-Domänen, Längenfelder und Byteordnung. Testkeys und Hashes sind künstlich. Vektoren prüfen Bytes und Semantik; sie beweisen weder Kryptografieimplementierung noch rechtliche Konformität.

## Schnittstellen und Abnahme

Der Generator schreibt in einen temporären Build-Ausgabepfad. Ein separater Vergleichstest liest die eingefrorenen Fixtures und vergleicht sie bytegenau mit dem Rust-Encoder/Decoder. Jede Änderung referenziert `SPEC.md`-Abschnitt, Besitzer-SP und Gate. Versionierung/Feature-Bits sind explizit; unbekannte Versionen werden abgelehnt, nicht geraten.

G-F bleibt offen, bis alle Formate der SPEC abgedeckt sind, Vektoren unabhängig geprüft wurden und Rust-Implementierung und Referenzcodierer dieselben positiven und negativen Fälle bestehen.
