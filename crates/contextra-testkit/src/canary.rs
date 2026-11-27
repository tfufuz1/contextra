//! # Canary-Prüfer für `T-DEL-1` (`canary`)
//!
//! Eindeutige Nadeln und ein byteweiser Scan eines Verzeichnisbaums. Der Scan ist
//! **fail-closed**: nicht lesbare Pfade sind ein Fehler, nie „0 Treffer“.
//!
//! Ein Scan ohne Treffer beweist nur dann etwas, wenn derselbe Scan *vor* dem Löschen
//! Treffer liefert (Negativkontrolle, Spezifikation 13 §7).

use crate::crash_vfs::XorShift;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Länge der Wertnadel in Bytes.
pub const VALUE_NEEDLE_LEN: usize = 48;

/// Nadeln eines Canary-Dokuments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canary {
    /// 48 eindeutige Bytes (druckbares ASCII, auch als Text-/Term-Inhalt verwendbar).
    pub value_needle: Vec<u8>,
    /// Eindeutiger Dokumentschlüssel.
    pub key_needle: Vec<u8>,
    /// `DocId` als LE, BE, Dezimal und Hex.
    pub doc_id_needles: Vec<Vec<u8>>,
}

impl Canary {
    /// Deterministisch aus `seed`; `doc_id` ist die vom System vergebene ID.
    ///
    /// Kleine `doc_id`s ergeben kurze Dezimalnadeln mit zufälligen Treffern; Tests
    /// sollten große IDs verwenden.
    pub fn from_seed(seed: u64, doc_id: u64) -> Self {
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = XorShift::new(seed);
        let mut token = |prefix: &str, len: usize| {
            let mut out = prefix.as_bytes().to_vec();
            while out.len() < len {
                let i = usize::try_from(rng.below(ALPHABET.len() as u64)).unwrap_or(0);
                out.push(ALPHABET.get(i).copied().unwrap_or(b'x'));
            }
            out
        };
        Self {
            value_needle: token("CNRY", VALUE_NEEDLE_LEN),
            key_needle: token("cnry-key-", 32),
            doc_id_needles: vec![
                doc_id.to_le_bytes().to_vec(),
                doc_id.to_be_bytes().to_vec(),
                doc_id.to_string().into_bytes(),
                format!("{doc_id:016x}").into_bytes(),
            ],
        }
    }
}

/// Optionen des Scans.
#[derive(Debug, Clone, Default)]
pub struct ScanOpts {
    /// Dokumentiert die Absicht; dateigestützte Mappings sind Dateiinhalt und werden
    /// ohnehin gelesen. Anonymer Speicher (Heap, Swap) wird **nie** durchsucht.
    pub include_mmap_regions: bool,
    /// Ausgenommene Pfade samt Unterbaum (nur externe Ledger-Anker).
    pub exclude: Vec<PathBuf>,
}

/// Fundstelle.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hit {
    /// Datei (bei Namenstreffern der Pfad selbst).
    pub path: PathBuf,
    /// Byteposition im Inhalt; `None` = Treffer im Dateinamen oder Linkziel.
    pub offset: Option<u64>,
    /// Index in `needles`.
    pub needle: usize,
}

const CHUNK: usize = 64 * 1024;

fn positions<'a>(hay: &'a [u8], needle: &'a [u8]) -> impl Iterator<Item = usize> + 'a {
    hay.windows(needle.len())
        .enumerate()
        .filter(move |(_, w)| *w == needle)
        .map(|(i, _)| i)
}

fn scan_name(path: &Path, name: &[u8], needles: &[&[u8]], hits: &mut Vec<Hit>) {
    for (i, needle) in needles.iter().enumerate() {
        if positions(name, needle).next().is_some() {
            hits.push(Hit {
                path: path.to_path_buf(),
                offset: None,
                needle: i,
            });
        }
    }
}

fn scan_file(path: &Path, needles: &[&[u8]], hits: &mut Vec<Hit>) -> io::Result<()> {
    let overlap = needles.iter().map(|n| n.len()).max().unwrap_or(1) - 1;
    let mut file = fs::File::open(path)?;
    let mut buf: Vec<u8> = Vec::with_capacity(CHUNK + overlap);
    let mut base = 0u64;
    loop {
        let carried = buf.len();
        let n = (&mut file).take(CHUNK as u64).read_to_end(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        for (i, needle) in needles.iter().enumerate() {
            for pos in positions(&buf, needle) {
                // Treffer vollständig im Übertrag wurden schon gemeldet.
                if pos + needle.len() > carried {
                    hits.push(Hit {
                        path: path.to_path_buf(),
                        offset: Some(base + pos as u64),
                        needle: i,
                    });
                }
            }
        }
        let drop = buf.len().saturating_sub(overlap);
        buf.drain(..drop);
        base += drop as u64;
    }
}

/// Durchsucht `root` rekursiv (Inhalte, Namen, Linkziele). Symlinks werden nicht verfolgt.
///
/// # Errors
/// Fehlende Wurzel, nicht lesbare Einträge, Spezialdateien, leere Nadelliste oder
/// leere Nadel.
pub fn scan_tree(root: &Path, needles: &[&[u8]], opts: ScanOpts) -> io::Result<Vec<Hit>> {
    if needles.is_empty() || needles.iter().any(|n| n.is_empty()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "needles must be non-empty",
        ));
    }
    let mut hits = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if opts.exclude.iter().any(|e| path.starts_with(e)) {
            continue;
        }
        let meta = fs::symlink_metadata(&path)?;
        if let Ok(rel) = path.strip_prefix(root) {
            if let Some(name) = rel.file_name() {
                scan_name(&path, name.as_encoded_bytes(), needles, &mut hits);
            }
        }
        if meta.is_dir() {
            for entry in fs::read_dir(&path)? {
                stack.push(entry?.path());
            }
        } else if meta.is_file() {
            scan_file(&path, needles, &mut hits)?;
        } else if meta.file_type().is_symlink() {
            let target = fs::read_link(&path)?;
            scan_name(
                &path,
                target.as_os_str().as_encoded_bytes(),
                needles,
                &mut hits,
            );
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("cannot scan special file {}", path.display()),
            ));
        }
    }
    hits.sort();
    Ok(hits)
}
