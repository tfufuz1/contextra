// FILE-CONTEXT
// STAND: 2026-09-27T22:49:25Z (SESSION: 72f4c80d)
// ZWECK: Zero-copy byte buffer slice wrapper (WireBuffer) for FlatBuffers IPC messages.
// INVARIANTEN: No pointer transmutes or lifetime erasures; reference counting via Bytes.
// SIEHE AUCH: docs/audits/contextra-wire_AUDIT_2026-09-27.md

//! # Zero-Copy-Adapter-Abstraktionen für IPC-Drahtnachrichten
//!
//! **Ring:** Ring 0
//! **Modulzweck:**
//! Dieses Modul definiert den `WireBuffer`, einen leichtgewichtigen Wrapper um `bytes::Bytes`.
//! Es ermöglicht das Weiterreichen von Byte-Puffern für FlatBuffers-Nachrichten über Modulgrenzen hinweg,
//! ohne dass Speicher kopiert werden muss (Zero-Copy). Durch die atomare Referenzzählung in `Bytes`
//! bleibt der Speicherzugriff threadsicher und allokationsarm.

use bytes::Bytes;

// ANCHOR[contextra-wire:ADP-001]
/// Zero-Copy Byte-Puffer-Wrapper für FlatBuffers-Nachrichten.
///
/// Kapselt eine Instanz von `bytes::Bytes`, um ein sicheres, referenzgezähltes Weiterreichen
/// von Binärdaten ohne Speicherallokation oder Kopiervorgänge zu ermöglichen.
///
/// # Invarianten
/// - Keine Zeiger-Transmutes (`transmute`) oder Lebenszeit-Löschungen (`lifetime erasures`).
/// - Threadsicher durch interne atomare Referenzzählung (`Arc`-äquivalent in `Bytes`).
#[derive(Debug, Clone)]
pub struct WireBuffer {
    data: Bytes,
}

impl WireBuffer {
    // ANCHOR[contextra-wire:ADP-002]
    /// Erzeugt einen neuen `WireBuffer`, der das übergebene `Bytes`-Objekt umhüllt.
    ///
    /// # Ablauf
    /// 1. Übernahme des `Bytes`-Objekts (Besitzübertragung ohne Kopiervorgang).
    /// 2. Rückgabe der initialisierten `WireBuffer`-Struktur.
    ///
    /// # Parameter
    /// - `data`: Das zu verpackende `Bytes`-Objekt.
    ///
    /// # Nebenwirkungen
    /// Keine. Keine I/O, keine Allokation.
    #[inline]
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }

    // ANCHOR[contextra-wire:ADP-003]
    /// Gibt eine unveränderliche Slice-Referenz auf die zugrundeliegenden Rohdaten zurück.
    ///
    /// # Ablauf
    /// 1. Zugriff auf den internen Byte-Puffer.
    /// 2. Rückgabe als Byte-Slice `&[u8]`.
    ///
    /// # Garantien
    /// Der Slice bleibt gültig, solange der `WireBuffer` existiert.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    // ANCHOR[contextra-wire:ADP-004]
    /// Konsumiert den Wrapper und gibt das zugrundeliegende `Bytes`-Objekt frei.
    ///
    /// # Ablauf
    /// 1. Entpacken des internen `data`-Felds.
    /// 2. Rückgabe des ursprünglichen `Bytes`-Objekts.
    #[inline]
    pub fn into_inner(self) -> Bytes {
        self.data
    }
}

// ANCHOR[contextra-wire:ADP-005]
// NOTE: Ermöglicht die Verwendung von `WireBuffer` überall dort, wo `AsRef<[u8]>` erwartet wird.
impl AsRef<[u8]> for WireBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
