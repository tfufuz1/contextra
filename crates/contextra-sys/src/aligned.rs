//! Safe 64-byte aligned buffer abstraction for kvcache and high-performance vector/cache I/O.
//!
//! Provides [`AlignedBuf`] which guarantees 64-byte boundary memory alignment
//! compatible with AVX-512, SIMD instructions, and cache line boundaries.

use std::alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc, Layout};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Alignment boundary in bytes (64 bytes / 512 bits).
pub const ALIGNMENT: usize = 64;

/// A 64-byte aligned byte buffer with safe slice access and dynamic capacity management.
pub struct AlignedBuf {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
}

// SAFETY: AlignedBuf owns its underlying memory uniquely and contains no thread-bound state.
unsafe impl Send for AlignedBuf {}

// SAFETY: Immutable concurrent references to AlignedBuf access read-only underlying buffer slices safely.
unsafe impl Sync for AlignedBuf {}

impl AlignedBuf {
    /// Creates a new empty [`AlignedBuf`] with zero capacity.
    #[inline]
    pub fn new() -> Self {
        Self {
            // SAFETY: 64 is a valid non-null aligned pointer constant for empty buffer sentinel.
            ptr: unsafe { NonNull::new_unchecked(ALIGNMENT as *mut u8) },
            len: 0,
            capacity: 0,
        }
    }

    /// Creates an empty [`AlignedBuf`] with at least `capacity` bytes pre-allocated on a 64-byte boundary.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buf = Self::new();
        if capacity > 0 {
            buf.reserve(capacity);
        }
        buf
    }

    /// Creates an [`AlignedBuf`] of length `len` initialized to zeros with 64-byte alignment.
    pub fn zeroed(len: usize) -> Self {
        if len == 0 {
            return Self::new();
        }

        let layout = match Layout::from_size_align(len, ALIGNMENT) {
            Ok(l) => l,
            Err(_) => return Self::new(),
        };

        // SAFETY: layout is valid and size > 0.
        let raw_ptr = unsafe { alloc_zeroed(layout) };
        let ptr = NonNull::new(raw_ptr).unwrap_or_else(|| handle_alloc_error(layout));

        Self {
            ptr,
            len,
            capacity: layout.size(),
        }
    }

    /// Creates an [`AlignedBuf`] initialized with a copy of `slice` content on 64-byte alignment.
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut buf = Self::with_capacity(slice.len());
        buf.extend_from_slice(slice);
        buf
    }

    /// Returns the number of initialized bytes in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no initialized bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total allocation capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the raw memory pointer to the underlying 64-byte aligned buffer.
    #[inline]
    pub fn as_raw_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    /// Returns the mutable raw memory pointer to the underlying 64-byte aligned buffer.
    #[inline]
    pub fn as_raw_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Returns a raw byte slice over the initialized buffer content.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr is non-null, 64-byte aligned, and points to len valid initialized bytes (or 0 bytes).
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns a mutable raw byte slice over the initialized buffer content.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: ptr is non-null, 64-byte aligned, and points to len valid initialized bytes (or 0 bytes).
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Appends the contents of `slice` to the end of the buffer.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        let additional = slice.len();
        if additional == 0 {
            return;
        }

        self.reserve(additional);

        // SAFETY: ptr is non-null and allocated with capacity >= self.len + additional.
        // The destination buffer area does not overlap with slice.
        unsafe {
            std::ptr::copy_nonoverlapping(
                slice.as_ptr(),
                self.ptr.as_ptr().add(self.len),
                additional,
            );
        }
        self.len += additional;
    }

    /// Ensures that capacity is at least `self.len() + additional` bytes.
    pub fn reserve(&mut self, additional: usize) {
        let required = match self.len.checked_add(additional) {
            Some(r) => r,
            None => return,
        };

        if required <= self.capacity {
            return;
        }

        let new_capacity = required
            .checked_next_power_of_two()
            .unwrap_or(required)
            .max(ALIGNMENT);

        let new_layout = match Layout::from_size_align(new_capacity, ALIGNMENT) {
            Ok(l) => l,
            Err(_) => return,
        };

        let new_ptr = if self.capacity == 0 {
            // SAFETY: new_layout has size > 0 and 64-byte alignment.
            let raw = unsafe { alloc(new_layout) };
            NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(new_layout))
        } else {
            let old_layout = match Layout::from_size_align(self.capacity, ALIGNMENT) {
                Ok(l) => l,
                Err(_) => return,
            };
            // SAFETY: self.ptr was allocated with old_layout and new_layout has size > 0.
            let raw = unsafe { realloc(self.ptr.as_ptr(), old_layout, new_layout.size()) };
            NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(new_layout))
        };

        self.ptr = new_ptr;
        self.capacity = new_layout.size();
    }

    /// Sets the length of the buffer.
    ///
    /// # Safety
    /// `new_len` must be less than or equal to `capacity()`, and the elements
    /// up to `new_len` must be initialized.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(
            new_len <= self.capacity,
            "set_len out of bounds ({new_len} > {})",
            self.capacity
        );
        self.len = new_len;
    }

    /// Clears the buffer, resetting length to 0 without freeing allocated memory.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Default for AlignedBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AlignedBuf {
    fn clone(&self) -> Self {
        Self::from_slice(self.as_slice())
    }
}

impl Deref for AlignedBuf {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for AlignedBuf {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        if self.capacity > 0 {
            if let Ok(layout) = Layout::from_size_align(self.capacity, ALIGNMENT) {
                // SAFETY: self.ptr was allocated with layout matching self.capacity and ALIGNMENT.
                unsafe {
                    dealloc(self.ptr.as_ptr(), layout);
                }
            }
        }
    }
}

impl fmt::Debug for AlignedBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlignedBuf")
            .field("address", &format_args!("{:p}", self.ptr.as_ptr()))
            .field("len", &self.len)
            .field("capacity", &self.capacity)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_buf_64byte_pointer_alignment() {
        let buf = AlignedBuf::with_capacity(128);
        assert_eq!(
            buf.as_raw_ptr() as usize % ALIGNMENT,
            0,
            "Address {:p} is not 64-byte aligned",
            buf.as_raw_ptr()
        );
        assert_eq!(
            buf.as_slice().as_ptr() as usize % ALIGNMENT,
            0,
            "Slice ptr is not 64-byte aligned"
        );
    }

    #[test]
    fn test_aligned_buf_zeroed() {
        let buf = AlignedBuf::zeroed(100);
        assert_eq!(buf.len(), 100);
        assert!(buf.capacity() >= 100);
        assert_eq!(
            buf.as_raw_ptr() as usize % ALIGNMENT,
            0,
            "Address is not 64-byte aligned"
        );
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_aligned_buf_extend_from_slice_and_deref() {
        let mut buf = AlignedBuf::new();
        let data = b"Hello, 64-byte aligned world!";
        buf.extend_from_slice(data);

        assert_eq!(buf.len(), data.len());
        assert_eq!(&buf[..], data);
        assert_eq!(buf.as_raw_ptr() as usize % ALIGNMENT, 0);

        buf[0] = b'h';
        assert_eq!(buf[0], b'h');
    }

    #[test]
    fn test_aligned_buf_clone() {
        let mut buf1 = AlignedBuf::with_capacity(32);
        buf1.extend_from_slice(b"aligned data");
        let buf2 = buf1.clone();

        assert_eq!(buf1.as_slice(), buf2.as_slice());
        assert_eq!(buf2.as_raw_ptr() as usize % ALIGNMENT, 0);
    }
}
