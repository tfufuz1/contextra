//! Integrationstest-Suite: `mod.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Minimal-Writer für OLE/CFB Test-Fixtures (Doc, Ppt, Msg).
// INVARIANTEN: Zero-Dependency CFB-Writer via cfb-Crate, deterministische Test-Bytes.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use cfb::CompoundFile;
use std::io::{Cursor, Write};

/// Erzeugt minimale MS-DOC CFB Bytes.
pub fn create_minimal_doc(
    ansi_text: &str,
    utf16_text: &str,
    field_code: &str,
    is_encrypted: bool,
) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut comp = CompoundFile::create(cursor).unwrap();

    let mut word_doc_stream = Vec::new();

    let mut fib = vec![0u8; 0x0200];
    fib[0..2].copy_from_slice(&0xA5ECu16.to_le_bytes());

    let flags: u16 = if is_encrypted { 0x0100 } else { 0x0000 };
    fib[0x000A..0x000C].copy_from_slice(&flags.to_le_bytes());

    let fib_header_len = 0x0200u32;
    word_doc_stream.extend_from_slice(&fib);

    let mut piece1_bytes = Vec::new();
    piece1_bytes.extend_from_slice(ansi_text.as_bytes());
    if !field_code.is_empty() {
        piece1_bytes.push(0x13);
        piece1_bytes.extend_from_slice(field_code.as_bytes());
        piece1_bytes.push(0x14);
        piece1_bytes.extend_from_slice(b"DisplayResult");
        piece1_bytes.push(0x15);
    }
    piece1_bytes.push(0x0D);

    let fc1 = fib_header_len;
    let len_cp1 = piece1_bytes.len() as u32;
    word_doc_stream.extend_from_slice(&piece1_bytes);

    let fc2_offset = word_doc_stream.len() as u32;
    let utf16_u16s: Vec<u16> = utf16_text.encode_utf16().collect();
    let mut piece2_bytes = Vec::new();
    for u in utf16_u16s {
        piece2_bytes.extend_from_slice(&u.to_le_bytes());
    }
    piece2_bytes.extend_from_slice(&0x000Du16.to_le_bytes());

    let len_cp2 = (piece2_bytes.len() / 2) as u32;
    word_doc_stream.extend_from_slice(&piece2_bytes);

    let total_cp = len_cp1 + len_cp2;
    fib[0x004C..0x0050].copy_from_slice(&total_cp.to_le_bytes());

    let fc_clx = 0u32;
    let cb_plc_pcd = (3 * 4 + 2 * 8) as u32;
    let lcb_clx = 1 + 4 + cb_plc_pcd;

    fib[0x01A2..0x01A6].copy_from_slice(&fc_clx.to_le_bytes());
    fib[0x01A6..0x01AA].copy_from_slice(&lcb_clx.to_le_bytes());

    word_doc_stream[0..0x0200].copy_from_slice(&fib);

    let mut table_stream = Vec::new();
    table_stream.push(0x02);
    table_stream.extend_from_slice(&cb_plc_pcd.to_le_bytes());

    table_stream.extend_from_slice(&0u32.to_le_bytes());
    table_stream.extend_from_slice(&len_cp1.to_le_bytes());
    table_stream.extend_from_slice(&(len_cp1 + len_cp2).to_le_bytes());

    table_stream.extend_from_slice(&0u16.to_le_bytes());
    let fc1_compressed = (fc1 * 2) | 0x4000_0000;
    table_stream.extend_from_slice(&fc1_compressed.to_le_bytes());
    table_stream.extend_from_slice(&0u16.to_le_bytes());

    table_stream.extend_from_slice(&0u16.to_le_bytes());
    table_stream.extend_from_slice(&fc2_offset.to_le_bytes());
    table_stream.extend_from_slice(&0u16.to_le_bytes());

    {
        let mut s = comp.create_stream("WordDocument").unwrap();
        s.write_all(&word_doc_stream).unwrap();
    }
    {
        let mut s = comp.create_stream("0Table").unwrap();
        s.write_all(&table_stream).unwrap();
    }

    comp.into_inner().into_inner()
}

/// Erzeugt minimale MS-PPT CFB Bytes.
pub fn create_minimal_ppt(
    slides: &[(&str, &str)],
    speaker_notes: &[&str],
    is_encrypted: bool,
) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut comp = CompoundFile::create(cursor).unwrap();

    let mut stream = Vec::new();

    if is_encrypted {
        let header = [0x0F, 0x00, 0x14, 0x2F, 0x08, 0x00, 0x00, 0x00];
        stream.extend_from_slice(&header);
        stream.extend_from_slice(&[0u8; 8]);
    } else {
        for (i, (title, body)) in slides.iter().enumerate() {
            let slide_num = (i + 1) as u32;

            let header_persist = [0x00, 0x00, 0xF0, 0x03, 0x04, 0x00, 0x00, 0x00];
            stream.extend_from_slice(&header_persist);
            stream.extend_from_slice(&slide_num.to_le_bytes());

            let header_slidelist = [0x0F, 0x00, 0xF5, 0x0F, 0x00, 0x00, 0x00, 0x00];
            let container_start = stream.len();
            stream.extend_from_slice(&header_slidelist);

            let header_th_title = [0x00, 0x00, 0xA8, 0x0F, 0x04, 0x00, 0x00, 0x00];
            stream.extend_from_slice(&header_th_title);
            stream.extend_from_slice(&0u32.to_le_bytes());

            let title_utf16: Vec<u8> = title.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
            let title_len = title_utf16.len() as u32;
            let header_tc_title = [
                0x00,
                0x00,
                0xA0,
                0x0F,
                title_len as u8,
                (title_len >> 8) as u8,
                (title_len >> 16) as u8,
                (title_len >> 24) as u8,
            ];
            stream.extend_from_slice(&header_tc_title);
            stream.extend_from_slice(&title_utf16);

            let header_th_body = [0x00, 0x00, 0xA8, 0x0F, 0x04, 0x00, 0x00, 0x00];
            stream.extend_from_slice(&header_th_body);
            stream.extend_from_slice(&1u32.to_le_bytes());

            let body_bytes = body.as_bytes();
            let body_len = body_bytes.len() as u32;
            let header_tb_body = [
                0x00,
                0x00,
                0xAB,
                0x0F,
                body_len as u8,
                (body_len >> 8) as u8,
                (body_len >> 16) as u8,
                (body_len >> 24) as u8,
            ];
            stream.extend_from_slice(&header_tb_body);
            stream.extend_from_slice(body_bytes);

            let payload_len = (stream.len() - container_start - 8) as u32;
            stream[container_start + 4..container_start + 8]
                .copy_from_slice(&payload_len.to_le_bytes());
        }

        for note in speaker_notes {
            let container_start = stream.len();
            let header_notes = [0x2F, 0x00, 0xF5, 0x0F, 0x00, 0x00, 0x00, 0x00];
            stream.extend_from_slice(&header_notes);

            let note_utf16: Vec<u8> = note.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
            let note_len = note_utf16.len() as u32;
            let header_tc_note = [
                0x00,
                0x00,
                0xA0,
                0x0F,
                note_len as u8,
                (note_len >> 8) as u8,
                (note_len >> 16) as u8,
                (note_len >> 24) as u8,
            ];
            stream.extend_from_slice(&header_tc_note);
            stream.extend_from_slice(&note_utf16);

            let payload_len = (stream.len() - container_start - 8) as u32;
            stream[container_start + 4..container_start + 8]
                .copy_from_slice(&payload_len.to_le_bytes());
        }
    }

    {
        let mut s = comp.create_stream("PowerPoint Document").unwrap();
        s.write_all(&stream).unwrap();
    }

    comp.into_inner().into_inner()
}

/// Erzeugt minimale MS-OXMSG CFB Bytes.
pub fn create_minimal_msg(
    subject: &str,
    sender: &str,
    body: &str,
    attach_name: Option<&str>,
    attach_data: Option<&[u8]>,
) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut comp = CompoundFile::create(cursor).unwrap();

    let subj_utf16: Vec<u8> = subject
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(|u| u.to_le_bytes())
        .collect();
    {
        let mut s = comp.create_stream("/__substg1.0_0037001F").unwrap();
        s.write_all(&subj_utf16).unwrap();
    }

    let sender_utf16: Vec<u8> = sender
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(|u| u.to_le_bytes())
        .collect();
    {
        let mut s = comp.create_stream("/__substg1.0_0C1A001F").unwrap();
        s.write_all(&sender_utf16).unwrap();
    }

    let body_utf16: Vec<u8> = body
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(|u| u.to_le_bytes())
        .collect();
    {
        let mut s = comp.create_stream("/__substg1.0_1000001F").unwrap();
        s.write_all(&body_utf16).unwrap();
    }

    let filetime: u64 = 133_748_544_000_000_000;
    {
        let mut s = comp.create_stream("/__substg1.0_00390040").unwrap();
        s.write_all(&filetime.to_le_bytes()).unwrap();
    }

    if let (Some(filename), Some(data)) = (attach_name, attach_data) {
        comp.create_storage("/__attach_version1.0_#00000000")
            .unwrap();

        {
            let mut s = comp
                .create_stream("/__attach_version1.0_#00000000/__substg1.0_37050003")
                .unwrap();
            s.write_all(&1u32.to_le_bytes()).unwrap();
        }

        let fn_utf16: Vec<u8> = filename
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(|u| u.to_le_bytes())
            .collect();
        {
            let mut s = comp
                .create_stream("/__attach_version1.0_#00000000/__substg1.0_3707001F")
                .unwrap();
            s.write_all(&fn_utf16).unwrap();
        }

        {
            let mut s = comp
                .create_stream("/__attach_version1.0_#00000000/__substg1.0_37010102")
                .unwrap();
            s.write_all(data).unwrap();
        }
    }

    comp.into_inner().into_inner()
}
