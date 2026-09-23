//! Generic zero-copy sub-slice backed by an `Arc<[T]>`.
//!
//! Provides lightweight, cheap-clone zero-copy sub-slicing without re-allocating
//! underlying elements, similar to `bytes::Bytes` for arbitrary types `T`.

use std::ops::{Deref, RangeBounds};
use std::sync::Arc;

/// A zero-copy slice view into an underlying `Arc<[T]>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcSlice<T> {
    shared: Arc<[T]>,
    offset: usize,
    len: usize,
}

impl<T> ArcSlice<T> {
    /// Creates a new `ArcSlice` wrapping the entire `Arc<[T]>`.
    #[inline]
    pub fn new(shared: Arc<[T]>) -> Self {
        let len = shared.len();
        Self {
            shared,
            offset: 0,
            len,
        }
    }

    /// Creates an `ArcSlice` from a `Vec<T>`.
    #[inline]
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self::new(vec.into())
    }

    /// Creates a zero-copy sub-slice within the given index range bounds.
    ///
    /// Returns `None` if the range bounds are invalid or out of bounds.
    pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Option<Self> {
        use std::ops::Bound;

        let start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s.checked_add(1)?,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&e) => e.checked_add(1)?,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.len,
        };

        if start > end || end > self.len {
            return None;
        }

        Some(Self {
            shared: Arc::clone(&self.shared),
            offset: self.offset + start,
            len: end - start,
        })
    }

    /// Creates a zero-copy sub-slice within the given index range bounds, clamping to valid bounds.
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        self.try_slice(range).unwrap_or_else(|| Self {
            shared: Arc::clone(&self.shared),
            offset: self.offset,
            len: 0,
        })
    }

    /// Returns a reference to the underlying `Arc<[T]>` slice.
    #[inline]
    pub fn as_arc(&self) -> &Arc<[T]> {
        &self.shared
    }

    /// Returns the offset within the backing `Arc<[T]>`.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the number of elements in the sub-slice.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the slice contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the slice of elements.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.shared[self.offset..self.offset + self.len]
    }
}

impl<T> Deref for ArcSlice<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> From<Arc<[T]>> for ArcSlice<T> {
    #[inline]
    fn from(arc: Arc<[T]>) -> Self {
        Self::new(arc)
    }
}

impl<T> From<Vec<T>> for ArcSlice<T> {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<'a, T> IntoIterator for &'a ArcSlice<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_slice_basic() {
        let vec = vec![10, 20, 30, 40, 50];
        let slice = ArcSlice::from_vec(vec);

        assert_eq!(slice.len(), 5);
        assert_eq!(&slice[..], &[10, 20, 30, 40, 50]);

        let sub = slice.slice(1..4);
        assert_eq!(sub.len(), 3);
        assert_eq!(&sub[..], &[20, 30, 40]);

        let sub2 = sub.slice(1..2);
        assert_eq!(sub2.len(), 1);
        assert_eq!(&sub2[..], &[30]);
    }
}
