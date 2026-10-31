//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// FILE-CONTEXT
// ZWECK: Defensives HTML/XHTML-Parsing, Boilerplate-Entfernung und IR-Block-Erzeugung.
// INVARIANTEN: Zero-Panic, iterativer Stack (keine Rekursion), iteratives Drop gegen Stack-Overflow, Fail-Closed, kein unsafe.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{
    BlockKind, DocMeta, DocumentIr, ExtractWarning, Locator, Table, WarningCode,
};
use html5ever::driver::ParseOpts;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{
    local_name, namespace_url, ns, parse_document, Attribute, ExpandedName, LocalName,
    Namespace, QualName,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

static HTML_NS: Namespace = ns!(html);
static EMPTY_LOCAL: LocalName = local_name!("");

/// Arten von Knoten im DOM-Baum.
#[derive(Debug)]
pub enum NodeKind {
    Document,
    Element {
        name: QualName,
        attrs: RefCell<Vec<Attribute>>,
    },
    Text(RefCell<String>),
    Comment(String),
    Doctype,
}

/// Knoten im DOM-Baum.
#[derive(Debug)]
pub struct Node {
    pub parent: RefCell<Option<Weak<Node>>>,
    pub children: RefCell<Vec<Rc<Node>>>,
    pub kind: NodeKind,
}

/// Speicher-Arena für den HTML-DOM-Baum.
#[derive(Debug)]
pub struct DomArena {
    pub document: Rc<Node>,
    pub quirks_mode: RefCell<QuirksMode>,
    pub max_depth_exceeded: RefCell<bool>,
    pub max_depth_limit: u32,
    pub observed_max_depth: RefCell<u32>,
}

impl DomArena {
    pub fn new(max_depth_limit: u32) -> Self {
        let root = Rc::new(Node {
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            kind: NodeKind::Document,
        });
        Self {
            document: root,
            quirks_mode: RefCell::new(QuirksMode::NoQuirks),
            max_depth_exceeded: RefCell::new(false),
            max_depth_limit,
            observed_max_depth: RefCell::new(0),
        }
    }

    fn check_node_depth(&self, node: &Rc<Node>) {
        let mut depth = 0u32;
        let mut curr = Some(node.clone());
        while let Some(n) = curr {
            depth = depth.saturating_add(1);
            let parent_weak = n.parent.borrow().clone();
            curr = parent_weak.and_then(|w| w.upgrade());
        }

        let mut obs = self.observed_max_depth.borrow_mut();
        if depth > *obs {
            *obs = depth;
        }

        if depth > self.max_depth_limit {
            *self.max_depth_exceeded.borrow_mut() = true;
        }
    }
}

impl TreeSink for DomArena {
    type Handle = Rc<Node>;
    type Output = DomArena;
    type ElemName<'a> = ExpandedName<'a>;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {}

    fn get_document(&self) -> Self::Handle {
        self.document.clone()
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        *self.quirks_mode.borrow_mut() = mode;
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        Rc::ptr_eq(x, y)
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        if let NodeKind::Element { ref name, .. } = target.kind {
            return name.expanded();
        }
        ExpandedName {
            ns: &HTML_NS,
            local: &EMPTY_LOCAL,
        }
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        target.clone()
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            NodeOrText::AppendNode(child_node) => {
                *child_node.parent.borrow_mut() = Some(Rc::downgrade(parent));
                parent.children.borrow_mut().push(child_node.clone());
                self.check_node_depth(&child_node);
            }
            NodeOrText::AppendText(text) => {
                let last_child = parent.children.borrow().last().cloned();
                if let Some(last) = last_child {
                    if let NodeKind::Text(ref existing) = last.kind {
                        existing.borrow_mut().push_str(text.as_ref());
                        return;
                    }
                }

                let text_node = Rc::new(Node {
                    parent: RefCell::new(Some(Rc::downgrade(parent))),
                    children: RefCell::new(Vec::new()),
                    kind: NodeKind::Text(RefCell::new(text.as_ref().to_string())),
                });
                parent.children.borrow_mut().push(text_node.clone());
                self.check_node_depth(&text_node);
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let parent = element.parent.borrow().as_ref().and_then(|w| w.upgrade());
        if let Some(p) = parent {
            self.append(&p, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let parent = sibling.parent.borrow().as_ref().and_then(|w| w.upgrade());
        if let Some(p) = parent {
            self.append(&p, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
        let dt = Rc::new(Node {
            parent: RefCell::new(Some(Rc::downgrade(&self.document))),
            children: RefCell::new(Vec::new()),
            kind: NodeKind::Doctype,
        });
        self.document.children.borrow_mut().push(dt);
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        if let NodeKind::Element {
            attrs: ref attrs_cell,
            ..
        } = target.kind
        {
            let mut existing = attrs_cell.borrow_mut();
            for attr in attrs {
                if !existing.iter().any(|e| e.name == attr.name) {
                    existing.push(attr);
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        let parent = target.parent.borrow().as_ref().and_then(|w| w.upgrade());
        if let Some(p) = parent {
            p.children.borrow_mut().retain(|c| !Rc::ptr_eq(c, target));
            *target.parent.borrow_mut() = None;
        }
    }

    fn reparent_children(&self, from: &Self::Handle, to: &Self::Handle) {
        let children = std::mem::take(&mut *from.children.borrow_mut());
        for child in &children {
            *child.parent.borrow_mut() = Some(Rc::downgrade(to));
        }
        to.children.borrow_mut().extend(children);
    }

    fn is_mathml_annotation_xml_integration_point(&self, _handle: &Self::Handle) -> bool {
        false
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        Rc::new(Node {
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            kind: NodeKind::Element {
                name,
                attrs: RefCell::new(attrs),
            },
        })
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        Rc::new(Node {
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            kind: NodeKind::Comment(text.as_ref().to_string()),
        })
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
        Rc::new(Node {
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            kind: NodeKind::Comment(String::new()),
        })
    }
}

/// Baut den Baum iterativ ab, um rekursiven Stack-Overflow beim Drop zu verhindern.
pub fn destroy_tree_iteratively(root: Rc<Node>) {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        let children = std::mem::take(&mut *node.children.borrow_mut());
        for child in children {
            *child.parent.borrow_mut() = None;
            stack.push(child);
        }
    }
}

/// Extrahiert HTML/XHTML-Inhalt defensiv in ein `DocumentIr`.
pub fn extract_html(
    html_text: &str,
    format: FormatId,
    ctx: &ExtractContext<'_>,
    is_guessed_encoding: bool,
) -> Result<DocumentIr, IngestError> {
    let mut ir = DocumentIr::new(format);

    if is_guessed_encoding {
        ir.warnings.push(ExtractWarning {
            code: WarningCode::EncodingGuessed,
            detail: "Textkodierung wurde geraten oder enthielt Ersetzungszeichen".into(),
        });
    }

    let sink = DomArena::new(ctx.limits.max_xml_depth);
    let arena = parse_document(sink, ParseOpts::default()).one(html_text);

    let max_exceeded = *arena.max_depth_exceeded.borrow();
    let max_observed = *arena.observed_max_depth.borrow();

    if max_exceeded {
        destroy_tree_iteratively(arena.document);
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::XmlDepth,
            limit: u64::from(ctx.limits.max_xml_depth),
            observed: u64::from(max_observed),
        });
    }

    let result = extract_from_arena(&arena, &mut ir, ctx);
    destroy_tree_iteratively(arena.document);

    result?;
    Ok(ir)
}

/// Extrahierung von Metadaten und Inhaltsblöcken aus der DOM-Arena.
fn extract_from_arena(
    arena: &DomArena,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    let has_main_or_article = scan_arena_structure(arena, &mut ir.meta, ctx)?;

    let mut builder = HtmlBlockBuilder::new(arena, ir, ctx, has_main_or_article);
    builder.traverse()?;

    Ok(())
}

/// Prüft die Arena auf Metadaten und ob ein <main>/<article> Bereich existiert.
fn scan_arena_structure(
    arena: &DomArena,
    meta: &mut DocMeta,
    ctx: &ExtractContext<'_>,
) -> Result<bool, IngestError> {
    let mut stack = vec![arena.document.clone()];
    let mut has_main = false;

    while let Some(node) = stack.pop() {
        if let NodeKind::Element { ref name, ref attrs } = node.kind {
            let tag_name = name.local.as_ref().to_ascii_lowercase();

            if tag_name == "main" || tag_name == "article" {
                has_main = true;
            } else if tag_name == "title" {
                let text = collect_node_text(&node);
                let clean = collapse_whitespace(&text);
                if !clean.is_empty() {
                    meta.title = Some(clean);
                }
            } else if tag_name == "meta" {
                let attrs_borrow = attrs.borrow();
                let mut meta_name = None;
                let mut meta_content = None;

                for attr in attrs_borrow.iter() {
                    let k = attr.name.local.as_ref().to_ascii_lowercase();
                    if k == "name" {
                        meta_name = Some(attr.value.as_ref().to_ascii_lowercase());
                    } else if k == "content" {
                        meta_content = Some(attr.value.as_ref().to_string());
                    }
                }

                if let (Some(m_name), Some(m_content)) = (meta_name, meta_content) {
                    if m_name == "description" {
                        meta.extra.insert("description".into(), m_content);
                    } else if m_name == "author" && ctx.policy.keep_personal_metadata {
                        meta.authors.push(m_content);
                    }
                }
            } else if tag_name == "html" {
                let attrs_borrow = attrs.borrow();
                for attr in attrs_borrow.iter() {
                    if attr.name.local.as_ref().eq_ignore_ascii_case("lang") {
                        meta.language = Some(attr.value.as_ref().to_string());
                    }
                }
            }
        }

        let children = node.children.borrow();
        for child in children.iter().rev() {
            stack.push(child.clone());
        }
    }

    Ok(has_main)
}

/// Sammelt Text aus allen Unterknoten eines Knoten.
fn collect_node_text(start: &Rc<Node>) -> String {
    let mut out = String::new();
    let mut stack = vec![start.clone()];

    while let Some(node) = stack.pop() {
        if let NodeKind::Text(ref contents) = node.kind {
            out.push_str(&contents.borrow());
        }
        let children = node.children.borrow();
        for child in children.iter().rev() {
            stack.push(child.clone());
        }
    }

    out
}

/// Kollabiert mehrfache Whitespace-Zeichen.
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_whitespace = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !in_whitespace {
                result.push(' ');
                in_whitespace = true;
            }
        } else {
            result.push(c);
            in_whitespace = false;
        }
    }

    result.trim().to_string()
}

enum TraversalOp {
    Enter(Rc<Node>),
    Exit(Rc<Node>),
}

struct HtmlBlockBuilder<'a, 'b> {
    arena: &'a DomArena,
    ir: &'a mut DocumentIr,
    ctx: &'b ExtractContext<'b>,
    has_main_or_article: bool,
    hidden_warning_added: bool,
    text_buffer: String,
    current_block_kind: Option<BlockTag>,
    list_stack: Vec<ListContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockTag {
    Heading(u8),
    Paragraph,
    ListItem,
    Code(Option<&'static str>),
    Quote,
    Caption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ListContext {
    ordered: bool,
    depth: u8,
}

impl<'a, 'b> HtmlBlockBuilder<'a, 'b> {
    fn new(
        arena: &'a DomArena,
        ir: &'a mut DocumentIr,
        ctx: &'b ExtractContext<'b>,
        has_main_or_article: bool,
    ) -> Self {
        Self {
            arena,
            ir,
            ctx,
            has_main_or_article,
            hidden_warning_added: false,
            text_buffer: String::new(),
            current_block_kind: None,
            list_stack: Vec::new(),
        }
    }

    fn traverse(&mut self) -> Result<(), IngestError> {
        let mut stack = vec![TraversalOp::Enter(self.arena.document.clone())];

        while let Some(op) = stack.pop() {
            match op {
                TraversalOp::Enter(node) => {
                    match node.kind {
                        NodeKind::Document => {
                            let children = node.children.borrow();
                            for child in children.iter().rev() {
                                stack.push(TraversalOp::Enter(child.clone()));
                            }
                        }
                        NodeKind::Element { ref name, ref attrs } => {
                            let tag = name.local.as_ref().to_ascii_lowercase();

                            if matches!(
                                tag.as_str(),
                                "script"
                                    | "style"
                                    | "noscript"
                                    | "template"
                                    | "iframe"
                                    | "svg"
                                    | "head"
                            ) {
                                continue;
                            }

                            let attrs_borrow = attrs.borrow();

                            if is_hidden_element(&tag, &attrs_borrow) && !self.ctx.policy.include_hidden {
                                if !self.hidden_warning_added {
                                    self.ir.warnings.push(ExtractWarning {
                                        code: WarningCode::HiddenContentSkipped,
                                        detail: "Versteckte HTML-Inhalte übersprungen".into(),
                                    });
                                    self.hidden_warning_added = true;
                                }
                                continue;
                            }

                            if self.has_main_or_article
                                && matches!(tag.as_str(), "nav" | "aside" | "header" | "footer")
                            {
                                continue;
                            }

                            if tag == "table" {
                                self.flush_text_block()?;
                                self.process_table(&node)?;
                                continue;
                            }

                            if tag == "img" {
                                for attr in attrs_borrow.iter() {
                                    if attr.name.local.as_ref().eq_ignore_ascii_case("alt") {
                                        let alt_text = attr.value.as_ref().trim();
                                        if !alt_text.is_empty() {
                                            self.flush_text_block()?;
                                            self.ctx.budget.charge_text(alt_text.len() as u64)?;
                                            self.ir.push(
                                                BlockKind::Caption {
                                                    text: alt_text.to_string(),
                                                },
                                                Locator::default(),
                                            );
                                        }
                                    }
                                }
                                continue;
                            }

                            if tag == "br" {
                                self.text_buffer.push('\n');
                                continue;
                            }

                            self.handle_element_enter(&tag, &attrs_borrow)?;

                            stack.push(TraversalOp::Exit(node.clone()));

                            let children = node.children.borrow();
                            for child in children.iter().rev() {
                                stack.push(TraversalOp::Enter(child.clone()));
                            }
                        }
                        NodeKind::Text(ref text) => {
                            self.text_buffer.push_str(&text.borrow());
                        }
                        _ => {}
                    }
                }
                TraversalOp::Exit(node) => {
                    if let NodeKind::Element { ref name, .. } = node.kind {
                        let tag = name.local.as_ref().to_ascii_lowercase();
                        self.handle_element_exit(&tag)?;
                    }
                }
            }
        }

        self.flush_text_block()?;
        Ok(())
    }

    fn handle_element_enter(&mut self, tag: &str, attrs: &[Attribute]) -> Result<(), IngestError> {
        match tag {
            "h1" => self.switch_block(BlockTag::Heading(1))?,
            "h2" => self.switch_block(BlockTag::Heading(2))?,
            "h3" => self.switch_block(BlockTag::Heading(3))?,
            "h4" => self.switch_block(BlockTag::Heading(4))?,
            "h5" => self.switch_block(BlockTag::Heading(5))?,
            "h6" => self.switch_block(BlockTag::Heading(6))?,
            "p" | "div" => {
                if self.current_block_kind.is_none() {
                    self.switch_block(BlockTag::Paragraph)?;
                }
            }
            "ul" => {
                self.flush_text_block()?;
                let depth = u8::try_from(self.list_stack.len()).unwrap_or(u8::MAX);
                self.list_stack.push(ListContext {
                    ordered: false,
                    depth,
                });
            }
            "ol" => {
                self.flush_text_block()?;
                let depth = u8::try_from(self.list_stack.len()).unwrap_or(u8::MAX);
                self.list_stack.push(ListContext {
                    ordered: true,
                    depth,
                });
            }
            "li" => {
                self.switch_block(BlockTag::ListItem)?;
            }
            "pre" | "code" => {
                let lang = extract_language(attrs);
                self.switch_block(BlockTag::Code(lang))?;
            }
            "blockquote" => self.switch_block(BlockTag::Quote)?,
            "figcaption" | "caption" => self.switch_block(BlockTag::Caption)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_element_exit(&mut self, tag: &str) -> Result<(), IngestError> {
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "div" | "li" | "pre" | "code"
            | "blockquote" | "figcaption" => {
                self.flush_text_block()?;
            }
            "ul" | "ol" => {
                self.flush_text_block()?;
                self.list_stack.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn switch_block(&mut self, new_tag: BlockTag) -> Result<(), IngestError> {
        self.flush_text_block()?;
        self.current_block_kind = Some(new_tag);
        Ok(())
    }

    fn flush_text_block(&mut self) -> Result<(), IngestError> {
        let text = collapse_whitespace(&self.text_buffer);
        self.text_buffer.clear();

        if text.is_empty() {
            self.current_block_kind = None;
            return Ok(());
        }

        self.ctx.budget.charge_text(text.len() as u64)?;

        let kind = match self.current_block_kind {
            Some(BlockTag::Heading(level)) => BlockKind::Heading { level, text },
            Some(BlockTag::Paragraph) => BlockKind::Paragraph { text },
            Some(BlockTag::ListItem) => {
                let ctx = self.list_stack.last().copied().unwrap_or(ListContext {
                    ordered: false,
                    depth: 0,
                });
                BlockKind::ListItem {
                    depth: ctx.depth,
                    ordered: ctx.ordered,
                    text,
                }
            }
            Some(BlockTag::Code(lang)) => BlockKind::Code {
                lang: lang.map(|s| s.to_string()),
                text,
            },
            Some(BlockTag::Quote) => BlockKind::Quote { text },
            Some(BlockTag::Caption) => BlockKind::Caption { text },
            None => BlockKind::Paragraph { text },
        };

        self.ir.push(kind, Locator::default());
        self.current_block_kind = None;
        Ok(())
    }

    fn process_table(&mut self, table_node: &Rc<Node>) -> Result<(), IngestError> {
        let mut caption = None;
        let mut header_rows = 0u8;
        let mut rows: Vec<Vec<CellData>> = Vec::new();

        let mut stack = vec![table_node.clone()];

        while let Some(node) = stack.pop() {
            if let NodeKind::Element { ref name, .. } = node.kind {
                let tag = name.local.as_ref().to_ascii_lowercase();

                if tag == "caption" {
                    let cap_text = collect_node_text(&node);
                    let clean = collapse_whitespace(&cap_text);
                    if !clean.is_empty() {
                        caption = Some(clean);
                    }
                    continue;
                }

                if tag == "tr" {
                    let mut row_cells = Vec::new();
                    if is_header_tr(&node) {
                        header_rows = header_rows.saturating_add(1);
                    }

                    extract_tr_cells(&node, &mut row_cells);
                    if !row_cells.is_empty() {
                        rows.push(row_cells);
                    }
                    continue;
                }
            }

            let children = node.children.borrow();
            for child in children.iter().rev() {
                stack.push(child.clone());
            }
        }

        if rows.is_empty() {
            return Ok(());
        }

        let flat_rows = flatten_table_grid(&rows)?;

        let mut total_cells = 0u64;
        for r in &flat_rows {
            total_cells = total_cells.saturating_add(r.len() as u64);
        }
        self.ctx.budget.charge_cells(total_cells)?;

        let table = Table {
            caption,
            header_rows,
            rows: flat_rows,
            origin: (0, 0),
        };

        self.ir.push(BlockKind::Table(table), Locator::default());
        Ok(())
    }
}

fn is_hidden_element(tag: &str, attrs: &[Attribute]) -> bool {
    for attr in attrs {
        let k = attr.name.local.as_ref().to_ascii_lowercase();
        if k == "hidden" {
            return true;
        }
        if k == "type" && tag == "input" && attr.value.as_ref().eq_ignore_ascii_case("hidden") {
            return true;
        }
        if k == "style" {
            let val = attr.value.as_ref().to_ascii_lowercase();
            let compact: String = val.chars().filter(|c| !c.is_whitespace()).collect();
            if compact.contains("display:none") || compact.contains("visibility:hidden") {
                return true;
            }
        }
    }
    false
}

fn extract_language(attrs: &[Attribute]) -> Option<&'static str> {
    for attr in attrs {
        if attr.name.local.as_ref().eq_ignore_ascii_case("class") {
            let val = attr.value.as_ref();
            for class_name in val.split_whitespace() {
                if let Some(lang) = class_name
                    .strip_prefix("language-")
                    .or_else(|| class_name.strip_prefix("lang-"))
                {
                    match lang.to_ascii_lowercase().as_str() {
                        "rust" => return Some("rust"),
                        "python" => return Some("python"),
                        "javascript" | "js" => return Some("javascript"),
                        "typescript" | "ts" => return Some("typescript"),
                        "c" => return Some("c"),
                        "cpp" | "c++" => return Some("cpp"),
                        "html" => return Some("html"),
                        "json" => return Some("json"),
                        "xml" => return Some("xml"),
                        "bash" | "sh" => return Some("bash"),
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

fn is_header_tr(tr_node: &Rc<Node>) -> bool {
    let mut stack = vec![tr_node.clone()];
    while let Some(node) = stack.pop() {
        if let NodeKind::Element { ref name, .. } = node.kind {
            if name.local.as_ref().eq_ignore_ascii_case("th") {
                return true;
            }
        }
        let children = node.children.borrow();
        for child in children.iter().rev() {
            stack.push(child.clone());
        }
    }
    false
}

fn extract_tr_cells(tr_node: &Rc<Node>, out_cells: &mut Vec<CellData>) {
    let mut stack = vec![tr_node.clone()];
    while let Some(node) = stack.pop() {
        if let NodeKind::Element {
            ref name,
            ref attrs,
        } = node.kind
        {
            let tag = name.local.as_ref().to_ascii_lowercase();
            if tag == "td" || tag == "th" {
                let cell_text = collect_node_text(&node);
                let clean = collapse_whitespace(&cell_text);

                let mut colspan = 1u32;
                let mut rowspan = 1u32;

                let attrs_borrow = attrs.borrow();
                for attr in attrs_borrow.iter() {
                    let k = attr.name.local.as_ref().to_ascii_lowercase();
                    if k == "colspan" {
                        colspan = attr.value.as_ref().parse::<u32>().unwrap_or(1).max(1);
                    } else if k == "rowspan" {
                        rowspan = attr.value.as_ref().parse::<u32>().unwrap_or(1).max(1);
                    }
                }

                out_cells.push(CellData {
                    text: clean,
                    colspan,
                    rowspan,
                });
                continue;
            }
        }

        let children = node.children.borrow();
        for child in children.iter().rev() {
            stack.push(child.clone());
        }
    }
}

struct CellData {
    text: String,
    colspan: u32,
    rowspan: u32,
}

fn flatten_table_grid(rows: &[Vec<CellData>]) -> Result<Vec<Vec<String>>, IngestError> {
    let mut grid: Vec<Vec<String>> = Vec::new();

    for (r_idx, row) in rows.iter().enumerate() {
        if grid.len() <= r_idx {
            grid.push(Vec::new());
        }

        let mut c_idx = 0usize;
        for cell in row {
            while grid
                .get(r_idx)
                .and_then(|r| r.get(c_idx))
                .is_some_and(|val| !val.is_empty())
            {
                c_idx = c_idx.saturating_add(1);
            }

            let span_rows = usize::try_from(cell.rowspan).unwrap_or(1);
            let span_cols = usize::try_from(cell.colspan).unwrap_or(1);

            for dr in 0..span_rows {
                let target_r = r_idx.saturating_add(dr);
                while grid.len() <= target_r {
                    grid.push(Vec::new());
                }

                for dc in 0..span_cols {
                    let target_c = c_idx.saturating_add(dc);
                    let target_row = grid.get_mut(target_r).ok_or_else(|| IngestError::Corrupt {
                        format: Some(FormatId::Html),
                        detail: "Ungültige Tabellenstruktur".into(),
                    })?;

                    while target_row.len() <= target_c {
                        target_row.push(String::new());
                    }

                    if let Some(slot) = target_row.get_mut(target_c) {
                        *slot = cell.text.clone();
                    }
                }
            }

            c_idx = c_idx.saturating_add(span_cols);
        }
    }

    Ok(grid)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;
    use crate::extractor::ResourceBudget;
    use crate::limits::{ExtractPolicy, IngestLimits};
    use crate::registry::ExtractorRegistry;

    fn make_ctx<'a>(
        limits: &'a IngestLimits,
        policy: &'a ExtractPolicy,
        budget: &'a ResourceBudget,
        reg: &'a ExtractorRegistry,
    ) -> ExtractContext<'a> {
        ExtractContext {
            limits,
            policy,
            budget,
            registry: reg,
            filename: None,
        }
    }

    #[test]
    fn test_extract_html_basic() {
        let limits = IngestLimits::default();
        let policy = ExtractPolicy::default();
        let budget = ResourceBudget::new(limits.clone());
        let reg = ExtractorRegistry::new_empty();
        let ctx = make_ctx(&limits, &policy, &budget, &reg);

        let html = "<html><head><title>Test Title</title></head><body><h1>Header</h1><p>Paragraph text</p></body></html>";
        let ir = extract_html(html, FormatId::Html, &ctx, false).unwrap();

        assert_eq!(ir.meta.title.as_deref(), Some("Test Title"));
        assert_eq!(ir.blocks.len(), 2);
        assert_eq!(
            ir.blocks.first().map(|b| &b.kind),
            Some(&BlockKind::Heading {
                level: 1,
                text: "Header".into()
            })
        );
        assert_eq!(
            ir.blocks.get(1).map(|b| &b.kind),
            Some(&BlockKind::Paragraph {
                text: "Paragraph text".into()
            })
        );
    }
}
