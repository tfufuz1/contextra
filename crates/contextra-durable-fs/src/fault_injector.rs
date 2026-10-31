//! # Fault-Injector-Subsystem für Dateisystem-Operationen
//!
//! Zweck: Bereitstellung von Schnittstellen zur gezielten Simulation von I/O-Fehlern (Input/Output)
//! in Persistenzpfaden (Schreibzugriffe, `fsync`-Synchronisationen, atomare Umbenennungen und Verzeichnis-Syncs).
//!
//! Ring-Zugehörigkeit: Ring 0 (Basisinfrastruktur / Systemabstraktion).
//! Wird genutzt von: `contextra-durable-fs` internen Operationen und externen Testsuiten zur Chaos- und Crash-Resilienzprüfung.
//! Wichtigste Typen: [`FaultInjector`] (Hook-Schnittstelle) und [`NoFault`] (Null-Kosten-Standardinjektor für die Produktion).
//!
//! Invarianten:
//! - In der Produktion darf die Fehlerinjektion keine Laufzeitkosten verursachen (Zero-Cost durch Monomorphisierung und Inlining via Generics).
//! - Kein `panic!` in Injektionspfaden; Fehler werden als deterministisches [`std::io::Result`] zurückgegeben.

// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Trait FaultInjector und NoFault Standard-Injektor fuer I/O-Fehlerinjektion in Durability-Pfade.
// INVARIANTEN: Zero-Cost in Produktionspfaden via Inlining und Generics (kein dyn Dispatch), Zero-Panic.

use std::path::Path;

// ANCHOR[contextra-durable-fs:FAULT-TRAIT-001]
/// Schnittstelle (Trait) zur Injektion von I/O-Fehlern in dauerhafte Dateisystem-Operationen.
///
/// Ermöglicht es Testsuiten, gezielt Ausfälle wie Speichermangel (`ENOSPC` = Error No Space),
/// Hardware-Sync-Abbrüche bei `fsync` oder Fehler bei atomaren Dateiumbenennungen zu simulieren,
/// um die Fail-Closed-Zustandsautomaten und Poisoning-Mechanismen zu verifizieren.
///
/// Alle Methoden besitzen Standardimplementierungen, die `Ok(())` (keine Aktion / kein Fehler) zurückgeben.
///
/// Vorbedingungen:
/// - Implementierungen müssen threadsicher (`Send + Sync`) sein.
///
/// Nachbedingungen:
/// - Gibt die Methode `Err(...)` zurück, bricht die aufrufende Dateisystem-Operation fail-closed ab.
pub trait FaultInjector: Send + Sync {
    // ANCHOR[contextra-durable-fs:FAULT-HOOK-WRITE]
    /// Hook vor oder während eines Dateischreibvorgangs.
    ///
    /// Ablauf:
    /// 1. Optionaler Zielpfad `_path` und geschriebene Byteanzahl `_bytes` übergeben.
    /// 2. Bei Rückgabe von `Err` wird das Schreiben abgebrochen.
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(std::io::Error)`, wenn ein simulierter I/O-Schreibfehler ausgelöst werden soll.
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }

    // ANCHOR[contextra-durable-fs:FAULT-HOOK-FSYNC]
    /// Hook vor dem Hardware-Flush (`fsync`) einer Datei.
    ///
    /// Ablauf:
    /// 1. Überprüfung des Dateipfads.
    /// 2. Rückgabe von `Err` führt unmittelbar zur Vergiftung (Poisoning) des [`crate::DurableHandle`].
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(std::io::Error)` zur Simulation eines Dateisync-Fehlers.
    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        Ok(())
    }

    // ANCHOR[contextra-durable-fs:FAULT-HOOK-RENAME]
    /// Hook vor einer atomaren Umbenennungsoperation (`rename`).
    ///
    /// Ablauf:
    /// 1. Quell- und Zielpfad werden übergeben.
    /// 2. Ein Rückgabefehler verhindert das Ausführen des echten Umbenennens im Dateisystem.
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(std::io::Error)` zur Simulation von Dateisystem-Fehlern beim atomaren Ersetzen.
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }

    // ANCHOR[contextra-durable-fs:FAULT-HOOK-DIR-FSYNC]
    /// Hook vor dem Hardware-Flush (`fsync`) eines Verzeichnisses.
    ///
    /// Ablauf:
    /// 1. Verzeichnispfad wird übergeben.
    /// 2. Dient der Simulation von Metadaten-Flush-Fehlern auf Verzeichnisebene.
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(std::io::Error)` bei simuliertem Verzeichnis-Sync-Fehler.
    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

// ANCHOR[contextra-durable-fs:NO-FAULT-001]
/// Null-Kosten-Standardinjektor (No-Op) für Produktivumgebungen.
///
/// Alle Methoden sind als `#[inline(always)]` markiert und liefern sofort `Ok(())`,
/// sodass der Compiler die Aufrufe vollständig wegoptimiert (Zero-Cost-Abstraktion).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoFault;

impl FaultInjector for NoFault {
    #[inline(always)]
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

// NOTE: Ermöglicht die Übergabe von Injektor-Referenzen an generische Funktionen ohne Ownership-Transfer.
impl<T: FaultInjector + ?Sized> FaultInjector for &T {
    #[inline]
    fn on_write(&self, path: Option<&Path>, bytes: usize) -> std::io::Result<()> {
        (**self).on_write(path, bytes)
    }

    #[inline]
    fn on_fsync(&self, path: Option<&Path>) -> std::io::Result<()> {
        (**self).on_fsync(path)
    }

    #[inline]
    fn on_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        (**self).on_rename(from, to)
    }

    #[inline]
    fn on_dir_fsync(&self, dir: &Path) -> std::io::Result<()> {
        (**self).on_dir_fsync(dir)
    }
}
