# contextra-erasure-spi: Agenten- und Entwicklerrichtlinien

Dieses Dokument definiert verbindliche Regeln für die Wartung und Weiterentwicklung von `contextra-erasure-spi`.

## 1. Architekturschranken
- **Ring:** Ring 0 (SPI-Schnittstellenschicht).
- **Abhängigkeiten (ausgehend):** Ausschließlich `contextra-types`, `blake3`, `serde`, `thiserror`.
- **Darf nie:** Datenhalter (`store`, `vector`, `text`, `graph`, `crypto`, `mvcc`) importieren.
- **Keine I/O- oder Async-Runtime-Zwänge:** Die Kernschnittstellen bleiben synchron und deterministisch.
- **Kein Unsafe:** `#![forbid(unsafe_code)]` ist strikt einzuhalten.

## 2. Invarianten
- `INV-WITNESS-MINT-1`: Der `LayerWitness` darf niemals öffentliche Konstruktoren oder Standardwerte exponieren, die eine Nachweiserzeugung ohne Messung erlauben.
- `INV-FAILCLOSED-1`: Fehlende Positivkontrolle oder unzureichende Scan-Einheiten führen stets zu einem Fehler, nie zu einem gültigen Witness.
