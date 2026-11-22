// FILE-CONTEXT
// ZWECK: Defensives EPUB-Parsing, DRM-Prüfung, Spine-Extraktion und Kapitel-Strukturierung.
// INVARIANTEN: Zero-Panic, nur get/checked_*, keine std::collections::{HashMap,HashSet} (BTreeMap/Vec).
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::formats::web::{encoding, html};
use crate::ir::{BlockKind, Boundary, DocumentIr, Locator};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Extrahiert ein EPUB-Dokument aus ZIP-Bytes in ein `DocumentIr`.
pub fn extract_epub(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    let mut zip = ZipArchive::new(Cursor::new(input)).map_err(|e| IngestError::Corrupt {
        format: Some(FormatId::Epub),
        detail: format!("Ungültiges ZIP-Archiv: {}", e),
    })?;

    // 1. DRM-Prüfung über META-INF/encryption.xml
    check_epub_encryption(&mut zip)?;

    // 2. Erfassen des OPF-Pfades aus META-INF/container.xml
    let opf_path = get_container_opf_path(&mut zip)?;

    // 3. Parsen der OPF-Datei
    let opf_bytes = read_zip_entry(&mut zip, &opf_path)?;
    let opf_dir = get_parent_dir(&opf_path);
    let opf_info = parse_opf(&opf_bytes, ctx)?;

    let mut doc_ir = DocumentIr::new(FormatId::Epub);
    doc_ir.meta.title = opf_info.title;
    doc_ir.meta.language = opf_info.language;
    doc_ir.meta.authors = opf_info.authors;

    // 4. Titel aus Inhaltsverzeichnis (NCX) lesen falls vorhanden
    let toc_titles = if let Some(ref ncx_id) = opf_info.toc_id {
        if let Some(ncx_href) = opf_info.manifest.get(ncx_id) {
            let ncx_full_path = resolve_relative_path(&opf_dir, ncx_href);
            if let Ok(ncx_bytes) = read_zip_entry(&mut zip, &ncx_full_path) {
                parse_ncx_titles(&ncx_bytes).unwrap_or_default()
            } else {
                BTreeMap::new()
            }
        } else {
            BTreeMap::new()
        }
    } else {
        BTreeMap::new()
    };

    // 5. Obergrenzen-Prüfung für die Anzahl von Spine-Kapiteln
    let spine_len = u64::try_from(opf_info.spine.len()).unwrap_or(u64::MAX);
    if spine_len > u64::from(ctx.limits.max_archive_entries) {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::ArchiveEntries,
            limit: u64::from(ctx.limits.max_archive_entries),
            observed: spine_len,
        });
    }

    // 6. Kapitel in Spine-Reihenfolge extrahieren
    for idref in &opf_info.spine {
        let href = match opf_info.manifest.get(idref) {
            Some(h) => h,
            None => continue,
        };

        // Ignoriere NCX / Navigationsdokument als Hauptinhalt
        if opf_info.toc_id.as_ref() == Some(idref) {
            continue;
        }

        let chapter_path = resolve_relative_path(&opf_dir, href);
        let chapter_bytes = match read_zip_entry(&mut zip, &chapter_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Kapiteltitel ermitteln
        let chapter_title = toc_titles
            .get(href)
            .cloned()
            .or_else(|| toc_titles.get(&chapter_path).cloned());

        // Boundary-Marker für das Kapitel einfügen
        doc_ir.push(
            BlockKind::Boundary(Boundary::Section {
                title: chapter_title,
            }),
            Locator::default(),
        );

        // HTML-Kapitel extrahieren
        let decoded = encoding::decode_html_bytes(&chapter_bytes);
        let mut chapter_ir =
            html::extract_html(&decoded.text, FormatId::Xhtml, ctx, decoded.is_guessed)?;

        // Blöcke und Warnungen übernehmen
        doc_ir.blocks.append(&mut chapter_ir.blocks);
        doc_ir.warnings.append(&mut chapter_ir.warnings);
    }

    Ok(doc_ir)
}

/// Prüft, ob DRM oder bösartige Verschlüsselung in `META-INF/encryption.xml` deklariert ist.
fn check_epub_encryption<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<(), IngestError> {
    let mut file = match zip.by_name("META-INF/encryption.xml") {
        Ok(f) => f,
        Err(_) => return Ok(()), // Keine encryption.xml vorhanden => Unverschlüsselt
    };

    let mut xml_bytes = Vec::new();
    if file.read_to_end(&mut xml_bytes).is_err() {
        return Ok(());
    }

    let mut reader = Reader::from_reader(Cursor::new(&xml_bytes));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"EncryptionMethod" {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"Algorithm" {
                            let algo = String::from_utf8_lossy(&attr.value);
                            // Erlaubte Schriftoberflächen-Obfuskation
                            if algo != "http://www.idpf.org/2008/embedding"
                                && algo != "http://ns.adobe.com/pdf/enc/art/2008/tf/10"
                            {
                                return Err(IngestError::Encrypted {
                                    format: Some(FormatId::Epub),
                                });
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}

/// Liest den relativen Pfad der OPF-Datei aus `META-INF/container.xml`.
fn get_container_opf_path<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<String, IngestError> {
    let bytes = read_zip_entry(zip, "META-INF/container.xml")?;
    let mut reader = Reader::from_reader(Cursor::new(&bytes));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"rootfile" {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"full-path" {
                            let path = String::from_utf8_lossy(&attr.value).to_string();
                            if !path.is_empty() {
                                return Ok(path);
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(IngestError::Corrupt {
                    format: Some(FormatId::Epub),
                    detail: format!("Fehler in container.xml: {}", e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Err(IngestError::Corrupt {
        format: Some(FormatId::Epub),
        detail: "Kein rootfile in META-INF/container.xml gefunden".into(),
    })
}

struct OpfInfo {
    title: Option<String>,
    language: Option<String>,
    authors: Vec<String>,
    manifest: BTreeMap<String, String>, // id -> href
    spine: Vec<String>,                 // idrefs
    toc_id: Option<String>,
}

/// Parst die OPF-Datei (Metadaten, Manifest, Spine).
fn parse_opf(bytes: &[u8], ctx: &ExtractContext<'_>) -> Result<OpfInfo, IngestError> {
    let mut reader = Reader::from_reader(Cursor::new(bytes));
    reader.config_mut().trim_text(true);

    let mut title = None;
    let mut language = None;
    let mut authors = Vec::new();
    let mut manifest = BTreeMap::new();
    let mut spine = Vec::new();
    let mut toc_id = None;

    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if tag == "spine" {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"toc" {
                            toc_id = Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
                current_tag = tag;
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.local_name();
                let tag = name.as_ref();
                if tag == b"item" {
                    let mut id = None;
                    let mut href = None;
                    let mut media_type = None;

                    for attr in e.attributes().flatten() {
                        let k = attr.key.local_name();
                        if k.as_ref() == b"id" {
                            id = Some(String::from_utf8_lossy(&attr.value).to_string());
                        } else if k.as_ref() == b"href" {
                            href = Some(String::from_utf8_lossy(&attr.value).to_string());
                        } else if k.as_ref() == b"media-type" {
                            media_type = Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }

                    if let (Some(item_id), Some(item_href)) = (id, href) {
                        manifest.insert(item_id.clone(), item_href);
                        if media_type.as_deref() == Some("application/x-dtbncx+xml")
                            && toc_id.is_none()
                        {
                            toc_id = Some(item_id);
                        }
                    }
                } else if tag == b"itemref" {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"idref" {
                            spine.push(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                if !text.is_empty() {
                    match current_tag.as_str() {
                        "title" => title = Some(text),
                        "language" => language = Some(text),
                        "creator" if ctx.policy.keep_personal_metadata => authors.push(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(IngestError::Corrupt {
                    format: Some(FormatId::Epub),
                    detail: format!("Fehler beim Parsen der OPF-Datei: {}", e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(OpfInfo {
        title,
        language,
        authors,
        manifest,
        spine,
        toc_id,
    })
}

/// Liest Kapiteltitel aus der NCX-Datei.
fn parse_ncx_titles(bytes: &[u8]) -> Result<BTreeMap<String, String>, IngestError> {
    let mut reader = Reader::from_reader(Cursor::new(bytes));
    reader.config_mut().trim_text(true);

    let mut titles = BTreeMap::new();
    let mut current_title = String::new();
    let mut in_text = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"text" {
                    in_text = true;
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                current_title = String::from_utf8_lossy(e.as_ref()).trim().to_string();
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"text" {
                    in_text = false;
                }
            }
            Ok(Event::Empty(ref e)) => {
                if e.local_name().as_ref() == b"content" {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"src" {
                            let src = String::from_utf8_lossy(&attr.value).to_string();
                            let clean_src = src.split('#').next().unwrap_or(&src).to_string();
                            if !current_title.is_empty() {
                                titles.insert(clean_src, current_title.clone());
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(titles)
}

/// Liest die Bytes einer Datei aus dem ZIP-Archiv.
fn read_zip_entry<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
    path: &str,
) -> Result<Vec<u8>, IngestError> {
    let mut file = zip.by_name(path).map_err(|_| IngestError::Corrupt {
        format: Some(FormatId::Epub),
        detail: format!("Datei {} nicht im EPUB-Archiv gefunden", path),
    })?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| IngestError::Corrupt {
            format: Some(FormatId::Epub),
            detail: format!("Fehler beim Lesen von {}: {}", path, e),
        })?;

    Ok(bytes)
}

/// Ermittelt das Elternverzeichnis eines Pfades.
fn get_parent_dir(path: &str) -> String {
    if let Some(pos) = path.rfind('/') {
        path.get(..pos).unwrap_or("").to_string()
    } else {
        String::new()
    }
}

/// Löst einen relativen Pfad gegenüber dem OPF-Verzeichnis auf.
fn resolve_relative_path(parent_dir: &str, relative_path: &str) -> String {
    if parent_dir.is_empty() {
        relative_path.to_string()
    } else {
        format!("{}/{}", parent_dir, relative_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_relative_path() {
        assert_eq!(resolve_relative_path("EPUB", "ch1.xhtml"), "EPUB/ch1.xhtml");
        assert_eq!(resolve_relative_path("", "ch1.xhtml"), "ch1.xhtml");
    }
}
