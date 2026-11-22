//! # Erase-Modul
//!
//! SPEC §14 (INV-ERA-1 bis INV-ERA-7, INV-SCAN-1): Physische und kryptografische Löschung.
//! - INV-ERA-1: Logische Atomarität: View >= S sieht den Bereich leer, View < S liefert Erased.
//! - INV-ERA-2: Nach erase_done(S) enthält kein lebender Run und kein lebendes Redo-Segment Einträge im Bereich.
//! - INV-ERA-3: Schlüssel vor Datei: Der Kern führt den Auftrag nie aus, solange der Schlüssel nicht vernichtet ist.
//! - INV-ERA-4: Nur Roll-forward: ein begonnener Erase wird nach jedem Absturz fortgesetzt oder erneut versucht.
//! - INV-ERA-5: Lesen unterhalb S liefert Erased, nie Daten, nie leer.
//! - INV-ERA-6: Nach erase_done(S) enthält keine lebende Blob-Generation einen Body-Record im gelöschten Bereich.
//! - INV-ERA-7: Körper einer vernichteten Domäne ist weder in Redo noch Blob ohne Schlüssel lesbar.
//! - INV-SCAN-1: scan_physical ändert nichts und gibt keine Inhalte preis.
