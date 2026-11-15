//! Kanonische Kodierung (Spec 0.3, 3.3): Big-Endian, Laengenfelder geprueft.
//!
//! `Reader` und `Writer` sind die einzige Stelle, an der Plattenformate und Protokollrahmen
//! Felder lesen und schreiben. Jede Laenge wird gegen eine Obergrenze und gegen den Rest der
//! Eingabe geprueft, **bevor** Speicher reserviert wird (Speicherbudget, 3.6).

use crate::error::BaseError;

/// Lesen aus einem Byteslice, strikt Big-Endian, jede Grenze geprueft.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Neuer Leser ueber `buf`.
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { rest: buf }
    }

    /// Anzahl der noch nicht gelesenen Bytes.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.rest.len()
    }

    /// Liest genau `n` Bytes.
    ///
    /// # Errors
    /// `Truncated`, wenn weniger als `n` Bytes uebrig sind.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], BaseError> {
        if n > self.rest.len() {
            return Err(BaseError::Truncated);
        }
        let (head, tail) = self.rest.split_at(n);
        self.rest = tail;
        Ok(head)
    }

    /// Liest ein Feld fester Groesse.
    ///
    /// # Errors
    /// `Truncated`, wenn weniger als `N` Bytes uebrig sind.
    pub fn array<const N: usize>(&mut self) -> Result<[u8; N], BaseError> {
        let mut out = [0_u8; N];
        out.copy_from_slice(self.take(N)?);
        Ok(out)
    }

    /// Liest ein `u8`.
    ///
    /// # Errors
    /// `Truncated` bei fehlendem Byte.
    pub fn u8(&mut self) -> Result<u8, BaseError> {
        Ok(u8::from_be_bytes(self.array()?))
    }

    /// Liest ein `u16` (Big-Endian).
    ///
    /// # Errors
    /// `Truncated` bei fehlenden Bytes.
    pub fn u16(&mut self) -> Result<u16, BaseError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    /// Liest ein `u32` (Big-Endian).
    ///
    /// # Errors
    /// `Truncated` bei fehlenden Bytes.
    pub fn u32(&mut self) -> Result<u32, BaseError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    /// Liest ein `u64` (Big-Endian).
    ///
    /// # Errors
    /// `Truncated` bei fehlenden Bytes.
    pub fn u64(&mut self) -> Result<u64, BaseError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    /// Liest ein `i32` (Big-Endian, Zweierkomplement), z. B. Kalendertage.
    ///
    /// # Errors
    /// `Truncated` bei fehlenden Bytes.
    pub fn i32(&mut self) -> Result<i32, BaseError> {
        Ok(i32::from_be_bytes(self.array()?))
    }

    /// Liest ein `i64` (Big-Endian, Zweierkomplement).
    ///
    /// # Errors
    /// `Truncated` bei fehlenden Bytes.
    pub fn i64(&mut self) -> Result<i64, BaseError> {
        Ok(i64::from_be_bytes(self.array()?))
    }

    /// Liest `[len u32][Daten]`. `max` ist die Obergrenze der Nutzlast.
    ///
    /// # Errors
    /// `LengthOutOfBounds`, wenn `len > max` oder `len` den Rest uebersteigt; `Truncated`,
    /// wenn das Laengenfeld fehlt. Es wird nichts allokiert.
    pub fn blob(&mut self, max: usize) -> Result<&'a [u8], BaseError> {
        let len = usize::try_from(self.u32()?).map_err(|_| BaseError::LengthOutOfBounds)?;
        if len > max || len > self.rest.len() {
            return Err(BaseError::LengthOutOfBounds);
        }
        self.take(len)
    }

    /// Schliesst das Lesen ab.
    ///
    /// # Errors
    /// `TrailingBytes`, wenn Bytes uebrig sind.
    pub fn finish(self) -> Result<(), BaseError> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(BaseError::TrailingBytes)
        }
    }
}

/// Schreiben in einen begrenzten Puffer, strikt Big-Endian.
#[derive(Debug, Clone)]
pub struct Writer {
    buf: Vec<u8>,
    max: usize,
}

impl Writer {
    /// Neuer Schreiber mit Obergrenze `max` fuer die Gesamtlaenge. Es wird nicht vorab
    /// reserviert; der Puffer waechst hoechstens bis `max`.
    #[must_use]
    pub fn with_limit(max: usize) -> Self {
        Self {
            buf: Vec::new(),
            max,
        }
    }

    /// Bisher geschriebene Laenge.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// `true`, wenn noch nichts geschrieben wurde.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Haengt rohe Bytes an.
    ///
    /// # Errors
    /// `LengthOutOfBounds`, wenn die Obergrenze ueberschritten wuerde; der Puffer bleibt
    /// dann unveraendert.
    pub fn put_raw(&mut self, bytes: &[u8]) -> Result<(), BaseError> {
        let new_len = self
            .buf
            .len()
            .checked_add(bytes.len())
            .ok_or(BaseError::LengthOutOfBounds)?;
        if new_len > self.max {
            return Err(BaseError::LengthOutOfBounds);
        }
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    /// Schreibt ein `u8`.
    ///
    /// # Errors
    /// `LengthOutOfBounds` bei Ueberschreitung der Obergrenze.
    pub fn put_u8(&mut self, v: u8) -> Result<(), BaseError> {
        self.put_raw(&v.to_be_bytes())
    }

    /// Schreibt ein `u16` (Big-Endian).
    ///
    /// # Errors
    /// `LengthOutOfBounds` bei Ueberschreitung der Obergrenze.
    pub fn put_u16(&mut self, v: u16) -> Result<(), BaseError> {
        self.put_raw(&v.to_be_bytes())
    }

    /// Schreibt ein `u32` (Big-Endian).
    ///
    /// # Errors
    /// `LengthOutOfBounds` bei Ueberschreitung der Obergrenze.
    pub fn put_u32(&mut self, v: u32) -> Result<(), BaseError> {
        self.put_raw(&v.to_be_bytes())
    }

    /// Schreibt ein `u64` (Big-Endian).
    ///
    /// # Errors
    /// `LengthOutOfBounds` bei Ueberschreitung der Obergrenze.
    pub fn put_u64(&mut self, v: u64) -> Result<(), BaseError> {
        self.put_raw(&v.to_be_bytes())
    }

    /// Schreibt ein `i32` (Big-Endian, Zweierkomplement).
    ///
    /// # Errors
    /// `LengthOutOfBounds` bei Ueberschreitung der Obergrenze.
    pub fn put_i32(&mut self, v: i32) -> Result<(), BaseError> {
        self.put_raw(&v.to_be_bytes())
    }

    /// Schreibt ein `i64` (Big-Endian, Zweierkomplement).
    ///
    /// # Errors
    /// `LengthOutOfBounds` bei Ueberschreitung der Obergrenze.
    pub fn put_i64(&mut self, v: i64) -> Result<(), BaseError> {
        self.put_raw(&v.to_be_bytes())
    }

    /// Schreibt `[len u32][Daten]`.
    ///
    /// # Errors
    /// `LengthOutOfBounds`, wenn `data` nicht in ein `u32` passt oder die Obergrenze
    /// ueberschritten wuerde; der Puffer bleibt dann unveraendert.
    pub fn put_blob(&mut self, data: &[u8]) -> Result<(), BaseError> {
        let len = u32::try_from(data.len()).map_err(|_| BaseError::LengthOutOfBounds)?;
        let total = data
            .len()
            .checked_add(4)
            .and_then(|n| n.checked_add(self.buf.len()))
            .ok_or(BaseError::LengthOutOfBounds)?;
        if total > self.max {
            return Err(BaseError::LengthOutOfBounds);
        }
        self.put_u32(len)?;
        self.put_raw(data)
    }

    /// Gibt den fertigen Puffer zurueck.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_are_big_endian() {
        let mut w = Writer::with_limit(64);
        w.put_u8(0x01).unwrap_or_default();
        w.put_u16(0x0203).unwrap_or_default();
        w.put_u32(0x0405_0607).unwrap_or_default();
        w.put_u64(0x0809_0a0b_0c0d_0e0f).unwrap_or_default();
        w.put_i32(-2).unwrap_or_default();
        assert_eq!(
            w.into_bytes(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0xff, 0xff, 0xff, 0xfe]
        );
    }

    #[test]
    fn roundtrip_all_types() -> Result<(), BaseError> {
        let mut w = Writer::with_limit(1024);
        w.put_u8(7)?;
        w.put_u16(65_000)?;
        w.put_u32(4_000_000_000)?;
        w.put_u64(u64::MAX - 3)?;
        w.put_i32(i32::MIN)?;
        w.put_i64(-5)?;
        w.put_blob(b"acta")?;
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        assert_eq!(r.u8()?, 7);
        assert_eq!(r.u16()?, 65_000);
        assert_eq!(r.u32()?, 4_000_000_000);
        assert_eq!(r.u64()?, u64::MAX - 3);
        assert_eq!(r.i32()?, i32::MIN);
        assert_eq!(r.i64()?, -5);
        assert_eq!(r.blob(16)?, b"acta");
        r.finish()
    }

    #[test]
    fn truncated_fields_fail() {
        let mut r = Reader::new(&[1, 2, 3]);
        assert_eq!(r.u32(), Err(BaseError::Truncated));
        // Fehlgeschlagenes Lesen verbraucht nichts.
        assert_eq!(r.remaining(), 3);
        assert_eq!(r.u16(), Ok(0x0102));
        assert_eq!(r.u16(), Err(BaseError::Truncated));
    }

    #[test]
    fn blob_length_is_checked_before_allocation() {
        // Laengenfeld behauptet 4 GiB, Rest hat 2 Bytes.
        let mut r = Reader::new(&[0xff, 0xff, 0xff, 0xff, 1, 2]);
        assert_eq!(r.blob(usize::MAX), Err(BaseError::LengthOutOfBounds));
        // Laenge passt zum Rest, ueberschreitet aber die Obergrenze.
        let mut r = Reader::new(&[0, 0, 0, 2, 1, 2]);
        assert_eq!(r.blob(1), Err(BaseError::LengthOutOfBounds));
        let mut r = Reader::new(&[0, 0, 0, 2, 1, 2]);
        assert_eq!(r.blob(2), Ok(&[1_u8, 2][..]));
        // Laengenfeld selbst fehlt.
        let mut r = Reader::new(&[0, 0]);
        assert_eq!(r.blob(8), Err(BaseError::Truncated));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut r = Reader::new(&[1, 2]);
        assert_eq!(r.u8(), Ok(1));
        assert_eq!(r.finish(), Err(BaseError::TrailingBytes));
    }

    #[test]
    fn writer_limit_is_enforced_and_atomic() {
        let mut w = Writer::with_limit(6);
        assert_eq!(w.put_u32(1), Ok(()));
        // Blob braucht 4 + 3 Bytes, es sind nur noch 2 frei: nichts wird geschrieben.
        assert_eq!(w.put_blob(b"abc"), Err(BaseError::LengthOutOfBounds));
        assert_eq!(w.len(), 4);
        assert_eq!(w.put_u16(2), Ok(()));
        assert_eq!(w.put_u8(3), Err(BaseError::LengthOutOfBounds));
        assert_eq!(w.len(), 6);
        assert!(!w.is_empty());
    }

    #[test]
    fn array_reads_fixed_fields() {
        let mut r = Reader::new(&[9, 8, 7, 6]);
        assert_eq!(r.array::<3>(), Ok([9, 8, 7]));
        assert_eq!(r.array::<2>(), Err(BaseError::Truncated));
    }
}
