//! Allokationsbudget des Heißpfads (SPEC §20.5, P-11): `seal_into` und `open_into`
//! allokieren bei vorab angelegtem `out` nicht. Gemessen wird mit einem zählenden
//! Allokator, der gegen die Methoden dieser Implementierung läuft, nicht gegen die
//! allokierende Standardmethode `seal`.
//!
//! Der Zähler ist pro Thread (thread-local), damit Allokationen des Test-Harness auf
//! anderen Threads die Messung nicht verfälschen.

#![allow(unsafe_code)] // Zählender GlobalAlloc: delegiert unverändert an System.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::error::Error;

use contextra_crypto::persistence::{XChaCha20Cipher, TAG_LEN};

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static ALLOCS: Cell<u64> = const { Cell::new(0) };
}

struct CountingAllocator;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn count_if_tracking() {
    // try_with: beim Thread-Ende ist der TLS-Zugriff nicht mehr möglich; dann nicht zählen.
    let _ = TRACKING.try_with(|t| {
        if t.get() {
            let _ = ALLOCS.try_with(|c| c.set(c.get() + 1));
        }
    });
}

// SAFETY: Jede Methode delegiert unverändert an `System`. Der Zähler nutzt nur
// thread-lokale `Cell`s ohne Heap-Allokation; Layouts reicht der Aufrufer gültig herein.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_if_tracking();
        // SAFETY: Layout kommt unverändert vom Aufrufer von `GlobalAlloc::alloc`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` und `layout` stammen aus einem Aufruf von `alloc` an `System`.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count_if_tracking();
        // SAFETY: `ptr`/`layout` stammen aus `System`; `new_size` ist vom Aufrufer gültig.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// Zählt Allokationen des aktuellen Threads während `f`.
fn allocations_during<F>(f: F) -> Result<u64, Box<dyn Error>>
where
    F: FnOnce() -> Result<(), Box<dyn Error>>,
{
    let _ = ALLOCS.try_with(|c| c.set(0));
    let _ = TRACKING.try_with(|t| t.set(true));
    let result = f();
    let _ = TRACKING.try_with(|t| t.set(false));
    let count = ALLOCS.try_with(|c| c.get()).unwrap_or(0);
    result?;
    Ok(count)
}

#[test]
fn counting_allocator_sees_allocating_default_method() -> Result<(), Box<dyn Error>> {
    // Kontrollmessung: Die allokierende Standardmethode muss gezählt werden, sonst wäre
    // der Nullbefund unten wertlos.
    let cipher = XChaCha20Cipher;
    let (key, nonce) = ([1u8; 32], [2u8; 24]);
    let plain = vec![3u8; 1024];
    let count = allocations_during(|| {
        let sealed = cipher.seal(&key, &nonce, b"", &plain);
        std::hint::black_box(sealed);
        Ok(())
    })?;
    assert!(
        count >= 1,
        "Zähler sieht keine Allokation (Zählung wirkungslos)"
    );
    Ok(())
}

#[test]
fn seal_into_and_open_into_do_not_allocate_with_prepared_buffers() -> Result<(), Box<dyn Error>> {
    let cipher = XChaCha20Cipher;
    let key = [7u8; 32];
    let nonce = [9u8; 24];
    let aad = b"ctxs-aad";
    let plain = vec![0x5Au8; 4096];

    // Heißpfad-Puffer werden vorab angelegt (SPEC §20.5).
    let mut sealed = Vec::with_capacity(plain.len() + TAG_LEN);
    let mut opened = Vec::with_capacity(plain.len() + TAG_LEN);

    let seal_allocs = allocations_during(|| {
        for _ in 0..1000 {
            cipher.seal_into(&key, &nonce, aad, &plain, &mut sealed)?;
        }
        Ok(())
    })?;
    let open_allocs = allocations_during(|| {
        for _ in 0..1000 {
            cipher.open_into(&key, &nonce, aad, &sealed, &mut opened)?;
        }
        Ok(())
    })?;

    assert_eq!(seal_allocs, 0, "seal_into allokiert im Heißpfad");
    assert_eq!(open_allocs, 0, "open_into allokiert im Heißpfad");
    assert_eq!(opened, plain);
    Ok(())
}
