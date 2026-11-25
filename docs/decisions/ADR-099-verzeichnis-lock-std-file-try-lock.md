# ADR-099: Verzeichnis-Locking via `std::fs::File::try_lock`

## Status
Proposed

## Kontext
Um Beschädigungen des LSM-Datenverzeichnisses zu verhindern, muss verhindert werden, dass zwei Instanzen von `LsmStorage` (entweder im selben Prozess oder in separaten Prozessen) gleichzeitig dasselbe Datenverzeichnis öffnen.

Bisher gab es im Subsystem `contextra-store` keine Verzeichnissperre. Externe Crate-Abhängigkeiten wie `fs4` oder `fd-lock` durften gemäß den Architekturregeln (keine neuen Abhängigkeiten in Ring-1) nicht hinzugefügt werden.

Seit Rust 1.89.0 sind die Methoden `try_lock`, `lock` und `unlock` auf `std::fs::File` in der Standardbibliothek stabilisiert.

## Entscheidung
Wir nutzen `std::fs::File::try_lock()` auf einer `LOCK`-Datei im Datenverzeichnis der Datenbank.

1. Beim Aufruf von `LsmStorage::new()` wird vor jedem Lesen oder Schreiben im Datenverzeichnis eine Datei `LOCK` geöffnet/erstellt und mittels `try_lock()` exklusiv gesperrt.
2. Der erhaltene File-Handle wird in der Datenstruktur `DirLock` gekapselt und an das `Manifest` gehängt, sodass die Sperre für die gesamte Lebensdauer der `LsmStorage`-Instanz gehalten wird.
3. Wenn `try_lock()` ein `TryLockError::WouldBlock` zurückgibt, bricht das Öffnen mit `ContextraError::Storage("Datenverzeichnis bereits in Benutzung")` ab.
4. Schlägt `try_lock()` aufgrund fehlender Sperr-Unterstützung des zugrundeliegenden Dateisystems fehl (z. B. manche NFS/Network-Mounts):
   - Im Modus `DurabilityMode::Full`: Abbruch mit Fehler `ContextraError::Storage("Dateisperre auf diesem Dateisystem nicht unterstützt")`.
   - In sonstigen Modi: Warnung per `tracing::warn!` und Fortfahren ohne Sperre.

## Konsequenzen
- **Sicherheit:** Verhindert zuverlässig gleichzeitige Zugriffe auf dasselbe Datenverzeichnis im selben Prozess sowie zwischen Prozessen.
- **Keine Abhängigkeiten:** Keine externen Crates (`fs4`, `fd-lock`) erforderlich; 100% klares Standard-Rust.
- **Grenzen:** Auf NFS/Netzwerkdateisystemen ohne Locking-Unterstützung wird die Sperre gemäß `DurabilityMode` behandelt.
