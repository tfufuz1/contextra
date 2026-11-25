// FILE-CONTEXT
// STAND: 2026-11-08T12:00:00Z
// ZWECK: Zero-copy byte buffer slice wrapper (WireBuffer) with typestate verification for FlatBuffers IPC messages.
// INVARIANTEN: No pointer transmutes or lifetime transmutations; reference counting via Bytes; INV-WIRE-BOUNDS-1 (Unverified -> Verified).

//! # Zero-Copy-Adapter-Abstraktionen für IPC-Drahtnachrichten
//!
//! **Ring:** Ring 0
//! **Modulzweck:**
//! Dieses Modul definiert den `WireBuffer`, einen leichtgewichtigen Wrapper um `bytes::Bytes`.
//! Es ermöglicht das Weiterreichen von Byte-Puffern für FlatBuffers-Nachrichten über Modulgrenzen hinweg,
//! ohne dass Speicher kopiert werden muss (Zero-Copy). Durch die atomare Referenzzählung in `Bytes`
//! bleibt der Speicherzugriff threadsicher und allokationsarm.
//!
//! ## Typzustand-Garantie (`INV-WIRE-BOUNDS-1`)
//! Gemäß `INV-WIRE-BOUNDS-1` gibt der Adapter keinen typisierten Zugriff auf unverifizierte Puffer heraus.
//! Ein neu erzeugter Puffer startet im Zustand `Unverified`. Erst durch explizite Verifikation
//! via `.verify::<T>(&config)` wird ein typisierter Puffer im Zustand `Verified<T>` erzeugt, der
//! sicheren Zugriff auf die geparste Wurzelstruktur gewährt.

use crate::verification::{verify_flatbuffer, FlatBufferConfig, FlatBufferError};
use bytes::Bytes;
use core::marker::PhantomData;

/// Typzustands-Marker: Unverifizierter Puffer.
///
/// Rohpuffer, dessen FlatBuffers-Offsets, VTables und Bounds noch nicht validiert wurden.
/// Aus diesem Zustand kann nicht direkt auf Root-Tabellen zugegriffen werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Unverified;

/// Typzustands-Marker: Verifizierter Puffer für FlatBuffers-Wurzeltabelle `T`.
///
/// Garantiert durch vorangegangene Verifikation gegen `FlatBufferConfig`, dass alle Offsets,
/// VTables, Vektoren und Bounds des Typs `T` innerhalb des Puffers liegen (`INV-WIRE-BOUNDS-1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Verified<T> {
    _phantom: PhantomData<fn() -> T>,
}

/// Helper trait providing higher-ranked follow and verification for root FlatBuffers tables.
pub trait VerifiableRoot {
    /// Associated inner table type with lifetime `'a`.
    type Table<'a>: flatbuffers::Follow<'a> + flatbuffers::Verifiable + 'a;
}

// Implement VerifiableRoot for all generated root tables
impl VerifiableRoot for crate::generated::contextra_generated::contextra::ipc::Embedding<'_> {
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::Embedding<'a>;
}

impl VerifiableRoot for crate::generated::contextra_generated::contextra::ipc::ScoredDocument<'_> {
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::ScoredDocument<'a>;
}

impl VerifiableRoot for crate::generated::contextra_generated::contextra::ipc::SearchResponse<'_> {
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::SearchResponse<'a>;
}

impl VerifiableRoot
    for crate::generated::contextra_generated::contextra::ipc::VectorIndexUpdate<'_>
{
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::VectorIndexUpdate<'a>;
}

impl VerifiableRoot for crate::generated::contextra_generated::contextra::ipc::RoleId<'_> {
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::RoleId<'a>;
}

impl VerifiableRoot for crate::generated::contextra_generated::contextra::ipc::HyperEdgeId<'_> {
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::HyperEdgeId<'a>;
}

impl VerifiableRoot for crate::generated::contextra_generated::contextra::ipc::RoleBinding<'_> {
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::RoleBinding<'a>;
}

impl VerifiableRoot for crate::generated::contextra_generated::contextra::ipc::HyperEdge<'_> {
    type Table<'a> = crate::generated::contextra_generated::contextra::ipc::HyperEdge<'a>;
}

// ANCHOR[contextra-wire:ADP-001]
/// Zero-Copy Byte-Puffer-Wrapper für FlatBuffers-Nachrichten mit Typzustand.
///
/// Kapselt eine Instanz von `bytes::Bytes`, um ein sicheres, referenzgezähltes Weiterreichen
/// von Binärdaten ohne Speicherallokation oder Kopiervorgänge zu ermöglichen.
///
/// # Invarianten
/// - `INV-WIRE-BOUNDS-1`: Typzustand `Unverified` -> `Verified<T>`. Wurzelzugriff existiert nur auf `Verified<T>`.
/// - Keine Zeiger-Transmutes (`transmute`) oder Lebenszeit-Manipulationen (`lifetime transmutations`).
/// - Threadsicher durch interne atomare Referenzzählung (`Arc`-äquivalent in `Bytes`).
#[derive(Debug, Clone)]
pub struct WireBuffer<State = Unverified> {
    data: Bytes,
    config: FlatBufferConfig,
    _state: PhantomData<State>,
}

impl WireBuffer<Unverified> {
    // ANCHOR[contextra-wire:ADP-002]
    /// Erzeugt einen neuen `WireBuffer` im Zustand `Unverified`, der das übergebene `Bytes`-Objekt umhüllt.
    ///
    /// # Ablauf
    /// 1. Übernahme des `Bytes`-Objekts (Besitzübertragung ohne Kopiervorgang).
    /// 2. Rückgabe der initialisierten `WireBuffer`-Struktur im Zustand `Unverified`.
    ///
    /// # Parameter
    /// - `data`: Das zu verpackende `Bytes`-Objekt.
    ///
    /// # Nebenwirkungen
    /// Keine. Keine I/O, keine Allokation.
    #[inline]
    pub fn new(data: Bytes) -> Self {
        Self {
            data,
            config: FlatBufferConfig::default(),
            _state: PhantomData,
        }
    }

    /// Verifiziert den Puffer gegen das angegebene FlatBuffers-Root-Schema `T` und die Schutzgrenzen `config`.
    ///
    /// # Ablauf (`INV-WIRE-BOUNDS-1`)
    /// 1. Übergabe des Rohdaten-Slices an `verification::verify_flatbuffer::<T::Table>`.
    /// 2. Bei Erfolg: Rückgabe eines typisierten `WireBuffer<Verified<T>>` mit gespeicherter Verifikationskonfiguration.
    /// 3. Bei Fehlschlag: Rückgabe von `FlatBufferError`.
    ///
    /// # Fehler
    /// Liefert `FlatBufferError::EmptyBuffer`, `FlatBufferError::BufferTooLarge` oder
    /// `FlatBufferError::VerificationFailed`, falls Grenzen oder Offsets verletzt sind.
    pub fn verify<T>(
        self,
        config: &FlatBufferConfig,
    ) -> Result<WireBuffer<Verified<T>>, FlatBufferError>
    where
        T: VerifiableRoot,
    {
        verify_flatbuffer::<T::Table<'_>>(&self.data, config)?;
        Ok(WireBuffer {
            data: self.data,
            config: config.clone(),
            _state: PhantomData,
        })
    }
}

impl<T: VerifiableRoot> WireBuffer<Verified<T>> {
    /// Liefert die verifizierte FlatBuffers-Wurzel für diesen Puffer unter Verwendung der bei der Verifikation geprüften Grenzwerte.
    ///
    /// # Garantien (`INV-WIRE-BOUNDS-1`)
    /// Durch den Typzustand `Verified<T>` ist sichergestellt, dass die Pufferstruktur,
    /// Offsets und Tabellengrenzen vorab vollständig geprüft wurden.
    ///
    /// # Fehler
    /// Liefert `FlatBufferError`, falls eine Re-Validierung wider Erwarten scheitert.
    #[inline]
    pub fn root<'a>(
        &'a self,
    ) -> Result<<T::Table<'a> as flatbuffers::Follow<'a>>::Inner, FlatBufferError> {
        self.root_with_config(&self.config)
    }

    /// Liefert die verifizierte FlatBuffers-Wurzel für diesen Puffer unter Verwendung der angegebenen Schutzgrenzen.
    ///
    /// # Garantien (`INV-WIRE-BOUNDS-1`)
    /// Sicherer, geprüfter Zugriff ohne ungeschützte Zeigerarithmetik.
    #[inline]
    pub fn root_with_config<'a>(
        &'a self,
        config: &FlatBufferConfig,
    ) -> Result<<T::Table<'a> as flatbuffers::Follow<'a>>::Inner, FlatBufferError> {
        verify_flatbuffer::<T::Table<'a>>(&self.data, config)
    }
}

impl<State> WireBuffer<State> {
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

    /// Gibt die Länge des Puffers in Bytes zurück.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Gibt an, ob der Puffer leer ist.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// ANCHOR[contextra-wire:ADP-005]
// NOTE: Ermöglicht die Verwendung von `WireBuffer` überall dort, wo `AsRef<[u8]>` erwartet wird.
impl<State> AsRef<[u8]> for WireBuffer<State> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
