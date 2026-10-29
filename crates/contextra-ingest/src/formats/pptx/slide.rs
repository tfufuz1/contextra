// FILE-CONTEXT
// ZWECK: Parser für einzelne Folien (ppt/slides/slideX.xml), Formen, Tabellen, Notizen & Kommentare.
// INVARIANTEN: Zero-Panic, ResourceBudget-Verbuchen, BTreeMap für Determinismus.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::sanitize_text;
use crate::error::IngestError;
use crate::extractor::ResourceBudget;
use crate::ir::{BlockKind, Boundary, ExtractWarning, Locator, NoteKind, Table, WarningCode};
use crate::limits::ExtractPolicy;
use crate::safe::archive::SafeZipReader;
use crate::safe::opc::{resolve_opc_target, OpcRels};
use crate::safe::xml::{parse_xml, XmlElement};

/// Ergebnis der Verarbeitung einer einzelnen Folie.
#[derive(Debug, Default)]
pub struct SlideExtractionResult {
    pub blocks: Vec<(BlockKind, Locator)>,
    pub warnings: Vec<ExtractWarning>,
}

/// Parst eine Folie, verarbeitet deren Formen, Tabellen, Notizen und Kommentare.
pub fn parse_slide_content(
    zip: &mut SafeZipReader<'_>,
    slide_path: &str,
    slide_num: u32,
    is_hidden_in_pres: bool,
    policy: &ExtractPolicy,
    budget: &ResourceBudget,
) -> Result<SlideExtractionResult, IngestError> {
    let mut result = SlideExtractionResult::default();
    let locator = Locator {
        slide: Some(slide_num),
        ..Default::default()
    };

    let slide_bytes = zip.read_file(slide_path, budget)?;
    let root = parse_xml(&slide_bytes, budget)?;

    // Prüfen, ob die Folie selbst show="0" hat
    let is_hidden =
        is_hidden_in_pres || root.attr("show").is_some_and(|v| v == "0" || v == "false");

    if is_hidden && !policy.include_hidden {
        result.warnings.push(ExtractWarning {
            code: WarningCode::HiddenContentSkipped,
            detail: format!("Folie {slide_num} ist ausgeblendet und wurde übersprungen"),
        });
        return Ok(result);
    }

    // Titel aus Platzhalter ctrTitle oder title ermitteln
    let title = extract_slide_title(&root);

    // Folien-Grenzblock emittieren
    result.blocks.push((
        BlockKind::Boundary(Boundary::Slide {
            number: slide_num,
            title: title.clone(),
        }),
        locator.clone(),
    ));

    // Formen-Baum (spTree) verarbeiten
    if let Some(sp_tree) = find_sp_tree(&root) {
        process_shape_container(
            sp_tree,
            zip,
            slide_path,
            &locator,
            policy,
            budget,
            &mut result,
        )?;
    }

    // Sprechernotizen und Kommentare über Folien-Beziehungen (.rels) suchen
    let slide_rels_path = get_rels_path(slide_path);
    if zip.contains_file(&slide_rels_path) {
        if let Ok(rels_bytes) = zip.read_file(&slide_rels_path, budget) {
            if let Ok(rels) = OpcRels::parse(&rels_bytes, budget) {
                // Sprechernotizen (notesSlide)
                for note_target in rels.find_targets_by_type_suffix("notesSlide") {
                    let note_path = resolve_opc_target(slide_path, note_target);
                    if let Ok(note_bytes) = zip.read_file(&note_path, budget) {
                        if let Ok(note_root) = parse_xml(&note_bytes, budget) {
                            let note_text = extract_notes_text(&note_root);
                            if !note_text.trim().is_empty() {
                                let (clean_text, _) = sanitize_text(&note_text);
                                result.blocks.push((
                                    BlockKind::Note {
                                        kind: NoteKind::SpeakerNotes,
                                        text: clean_text,
                                    },
                                    locator.clone(),
                                ));
                            }
                        }
                    }
                }

                // Kommentare (comments)
                if policy.include_comments {
                    for comment_target in rels.find_targets_by_type_suffix("comments") {
                        let comment_path = resolve_opc_target(slide_path, comment_target);
                        if let Ok(comment_bytes) = zip.read_file(&comment_path, budget) {
                            if let Ok(comment_root) = parse_xml(&comment_bytes, budget) {
                                for comment_text in extract_comments_text(&comment_root) {
                                    if !comment_text.trim().is_empty() {
                                        let (clean_text, _) = sanitize_text(&comment_text);
                                        result.blocks.push((
                                            BlockKind::Note {
                                                kind: NoteKind::Comment,
                                                text: clean_text,
                                            },
                                            locator.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

fn get_rels_path(path: &str) -> String {
    if let Some((dir, file)) = path.rsplit_once('/') {
        format!("{dir}/_rels/{file}.rels")
    } else {
        format!("_rels/{path}.rels")
    }
}

fn find_sp_tree(root: &XmlElement) -> Option<&XmlElement> {
    if root.local_name() == "spTree" {
        return Some(root);
    }
    for child in root.child_elements() {
        if child.local_name() == "cSld" {
            if let Some(sp_tree) = child.find_first_child("spTree") {
                return Some(sp_tree);
            }
        }
        if let Some(found) = find_sp_tree(child) {
            return Some(found);
        }
    }
    None
}

fn extract_slide_title(root: &XmlElement) -> Option<String> {
    let sp_tree = find_sp_tree(root)?;
    find_title_in_container(sp_tree)
}

fn find_title_in_container(container: &XmlElement) -> Option<String> {
    for child in container.child_elements() {
        let name = child.local_name();
        if name == "sp" {
            if is_placeholder_type(child, &["title", "ctrTitle"]) {
                if let Some(tx_body) = child.find_first_child("txBody") {
                    let text = collect_tx_body_text(tx_body);
                    if !text.trim().is_empty() {
                        let (clean, _) = sanitize_text(text.trim());
                        return Some(clean);
                    }
                }
            }
        } else if name == "grpSp" {
            if let Some(t) = find_title_in_container(child) {
                return Some(t);
            }
        }
    }
    None
}

fn is_placeholder_type(sp_elem: &XmlElement, target_types: &[&str]) -> bool {
    let nv_sp_pr = match sp_elem.find_first_child("nvSpPr") {
        Some(elem) => elem,
        None => return false,
    };
    let nv_pr = match nv_sp_pr.find_first_child("nvPr") {
        Some(elem) => elem,
        None => return false,
    };
    let ph = match nv_pr.find_first_child("ph") {
        Some(elem) => elem,
        None => return false,
    };

    if let Some(ph_type) = ph.attr("type") {
        target_types.contains(&ph_type)
    } else {
        false
    }
}

fn find_tbl_in_graphic_frame(frame: &XmlElement) -> Option<&XmlElement> {
    if let Some(graphic) = frame.find_first_child("graphic") {
        if let Some(graphic_data) = graphic.find_first_child("graphicData") {
            return graphic_data.find_first_child("tbl");
        }
    }
    None
}

#[allow(clippy::only_used_in_recursion)]
fn process_shape_container(
    container: &XmlElement,
    zip: &mut SafeZipReader<'_>,
    slide_path: &str,
    locator: &Locator,
    policy: &ExtractPolicy,
    budget: &ResourceBudget,
    result: &mut SlideExtractionResult,
) -> Result<(), IngestError> {
    for child in container.child_elements() {
        let name = child.local_name();
        match name {
            "sp" => {
                // Platzhalter prüfen: Foliennummer, Fußzeile, Datum überspringen
                if is_placeholder_type(child, &["sldNum", "ftr", "dt"]) {
                    continue;
                }
                // Titel-Platzhalter nicht als doppelten Absatz emittieren, da in Boundary bereits enthalten
                if is_placeholder_type(child, &["title", "ctrTitle"]) {
                    continue;
                }

                if let Some(tx_body) = child.find_first_child("txBody") {
                    process_tx_body(tx_body, locator, budget, &mut result.blocks)?;
                }
            }
            "grpSp" => {
                // Gruppenformen rekursiv auflösen
                process_shape_container(child, zip, slide_path, locator, policy, budget, result)?;
            }
            "graphicFrame" => {
                // Tabelle oder Diagramm oder OLE
                if let Some(tbl) = find_tbl_in_graphic_frame(child) {
                    process_table(tbl, locator, budget, &mut result.blocks)?;
                } else if let Some(graphic) = child.find_first_child("graphic") {
                    if let Some(graphic_data) = graphic.find_first_child("graphicData") {
                        let uri = graphic_data.attr("uri").unwrap_or("");
                        if uri.contains("chart") || graphic_data.find_first_child("chart").is_some()
                        {
                            // Diagramm: Text lesen
                            let chart_text = graphic_data.collect_text();
                            if !chart_text.trim().is_empty() {
                                let (clean, _) = sanitize_text(chart_text.trim());
                                result
                                    .blocks
                                    .push((BlockKind::Caption { text: clean }, locator.clone()));
                            }
                        } else if uri.contains("diagram")
                            || graphic_data.find_first_child("relIds").is_some()
                        {
                            // SmartArt: Text lesen
                            let smart_text = graphic_data.collect_text();
                            if !smart_text.trim().is_empty() {
                                let (clean, _) = sanitize_text(smart_text.trim());
                                result
                                    .blocks
                                    .push((BlockKind::Paragraph { text: clean }, locator.clone()));
                            }
                        }
                    }
                }
                if child.find_first_child("oleObj").is_some()
                    || child.collect_text().contains("oleObj")
                {
                    result.warnings.push(ExtractWarning {
                        code: WarningCode::EmbeddedObjectSkipped,
                        detail: "Eingebettetes OLE-Objekt übersprungen".into(),
                    });
                }
            }
            "pic" => {
                // Bild: Alt-Text (p:cNvPr descr="...") als Caption emittieren
                if let Some(nv_pic_pr) = child.find_first_child("nvPicPr") {
                    if let Some(c_nv_pr) = nv_pic_pr.find_first_child("cNvPr") {
                        let alt_text = c_nv_pr
                            .attr("descr")
                            .or_else(|| c_nv_pr.attr("title"))
                            .unwrap_or("");
                        if !alt_text.trim().is_empty() {
                            let (clean, _) = sanitize_text(alt_text.trim());
                            result
                                .blocks
                                .push((BlockKind::Caption { text: clean }, locator.clone()));
                        }
                    }
                }
            }
            "oleObj" => {
                result.warnings.push(ExtractWarning {
                    code: WarningCode::EmbeddedObjectSkipped,
                    detail: "Eingebettetes Objekt übersprungen".into(),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn process_tx_body(
    tx_body: &XmlElement,
    locator: &Locator,
    budget: &ResourceBudget,
    blocks: &mut Vec<(BlockKind, Locator)>,
) -> Result<(), IngestError> {
    for p_elem in tx_body.find_children("p") {
        let (p_text, lvl, is_bullet, is_ordered) = parse_paragraph(p_elem);
        if p_text.trim().is_empty() {
            continue;
        }

        budget.charge_text(p_text.len() as u64)?;
        let (clean_text, _) = sanitize_text(p_text.trim());

        if is_bullet || lvl.is_some() {
            let depth = lvl.unwrap_or(0);
            blocks.push((
                BlockKind::ListItem {
                    depth,
                    ordered: is_ordered,
                    text: clean_text,
                },
                locator.clone(),
            ));
        } else {
            blocks.push((BlockKind::Paragraph { text: clean_text }, locator.clone()));
        }
    }
    Ok(())
}

fn parse_paragraph(p_elem: &XmlElement) -> (String, Option<u8>, bool, bool) {
    let mut text_buf = String::new();
    let mut lvl: Option<u8> = None;
    let mut is_bullet = false;
    let mut is_ordered = false;

    if let Some(p_pr) = p_elem.find_first_child("pPr") {
        if let Some(lvl_str) = p_pr.attr("lvl") {
            lvl = lvl_str.parse::<u8>().ok();
        }
        if p_pr.find_first_child("buChar").is_some() || p_pr.find_first_child("buFont").is_some() {
            is_bullet = true;
        }
        if p_pr.find_first_child("buAutoNum").is_some() {
            is_bullet = true;
            is_ordered = true;
        }
    }

    for child in p_elem.child_elements() {
        if child.local_name() == "r" {
            for r_child in child.child_elements() {
                if r_child.local_name() == "t" {
                    text_buf.push_str(&r_child.collect_text());
                }
            }
        } else if child.local_name() == "fld" {
            if is_placeholder_type(child, &["sldNum", "ftr", "dt"]) {
                continue;
            }
            for fld_child in child.child_elements() {
                if fld_child.local_name() == "t" {
                    text_buf.push_str(&fld_child.collect_text());
                }
            }
        }
    }

    (text_buf, lvl, is_bullet, is_ordered)
}

fn process_table(
    tbl_elem: &XmlElement,
    locator: &Locator,
    budget: &ResourceBudget,
    blocks: &mut Vec<(BlockKind, Locator)>,
) -> Result<(), IngestError> {
    let header_rows = if let Some(tbl_pr) = tbl_elem.find_first_child("tblPr") {
        if tbl_pr.attr("firstRow") == Some("1") || tbl_pr.attr("firstRow") == Some("true") {
            1
        } else {
            0
        }
    } else {
        0
    };

    let mut rows = Vec::new();
    let mut total_cells: u64 = 0;

    for tr_elem in tbl_elem.find_children("tr") {
        let mut row_cells = Vec::new();
        for tc_elem in tr_elem.find_children("tc") {
            total_cells = total_cells.saturating_add(1);
            let mut cell_text = String::new();
            if let Some(tx_body) = tc_elem.find_first_child("txBody") {
                for p_elem in tx_body.find_children("p") {
                    let (p_text, _, _, _) = parse_paragraph(p_elem);
                    if !p_text.trim().is_empty() {
                        if !cell_text.is_empty() {
                            cell_text.push(' ');
                        }
                        cell_text.push_str(p_text.trim());
                    }
                }
            }
            let (clean_cell, _) = sanitize_text(&cell_text);
            row_cells.push(clean_cell);
        }
        rows.push(row_cells);
    }

    budget.charge_cells(total_cells)?;

    let table = Table {
        caption: None,
        header_rows,
        rows,
        origin: (0, 0),
    };

    blocks.push((BlockKind::Table(table), locator.clone()));
    Ok(())
}

fn collect_tx_body_text(tx_body: &XmlElement) -> String {
    let mut buf = String::new();
    for p_elem in tx_body.find_children("p") {
        let (p_text, _, _, _) = parse_paragraph(p_elem);
        if !p_text.trim().is_empty() {
            if !buf.is_empty() {
                buf.push(' ');
            }
            buf.push_str(p_text.trim());
        }
    }
    buf
}

fn extract_notes_text(root: &XmlElement) -> String {
    let mut buf = String::new();
    if let Some(sp_tree) = find_sp_tree(root) {
        for sp in sp_tree.find_children("sp") {
            if is_placeholder_type(sp, &["sldNum", "ftr", "dt"]) {
                continue;
            }
            if let Some(tx_body) = sp.find_first_child("txBody") {
                let text = collect_tx_body_text(tx_body);
                if !text.trim().is_empty() {
                    if !buf.is_empty() {
                        buf.push('\n');
                    }
                    buf.push_str(text.trim());
                }
            }
        }
    }
    buf
}

fn extract_comments_text(root: &XmlElement) -> Vec<String> {
    let mut comments = Vec::new();
    for cm in root.find_children("cm") {
        if let Some(text_elem) = cm.find_first_child("text") {
            let text = text_elem.collect_text();
            if !text.trim().is_empty() {
                comments.push(text.trim().to_string());
            }
        }
    }
    if comments.is_empty() {
        let text = root.collect_text();
        if !text.trim().is_empty() {
            comments.push(text.trim().to_string());
        }
    }
    comments
}
