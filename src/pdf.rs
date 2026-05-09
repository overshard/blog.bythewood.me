use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{Datelike, Local};
use comrak::{
    nodes::{AstNode, NodeValue, TableAlignment},
    Arena, Options,
};
use typst::{
    diag::{FileError, FileResult, SourceDiagnostic},
    foundations::{Bytes, Datetime},
    layout::PagedDocument,
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, LibraryExt, World,
};
use typst_kit::fonts::{FontSearcher, FontSlot, Fonts};

use crate::posts::Post;

/// Pre-built renderer state. Fonts and the standard library are loaded once at
/// startup and shared across renders.
pub struct PdfRenderer {
    library: Arc<LazyHash<Library>>,
    book: Arc<LazyHash<FontBook>>,
    fonts: Arc<Vec<FontSlot>>,
    root: PathBuf,
}

impl PdfRenderer {
    /// Discover system + embedded fonts and build the renderer. `root` is the
    /// project root that absolute paths in the Typst source resolve against
    /// (e.g. `image("/content/images/foo.webp")` → `<root>/content/images/foo.webp`).
    pub fn new(root: PathBuf) -> Self {
        let Fonts { book, fonts } = FontSearcher::new()
            .include_system_fonts(true)
            .search();
        Self {
            library: Arc::new(LazyHash::new(Library::default())),
            book: Arc::new(LazyHash::new(book)),
            fonts: Arc::new(fonts),
            root,
        }
    }

    /// Compile `source` (Typst markup) into a PDF.
    pub fn render(&self, source: String) -> anyhow::Result<Vec<u8>> {
        let main_id = FileId::new(None, VirtualPath::new("/main.typ"));
        let main = Source::new(main_id, source);
        let world = PdfWorld {
            library: self.library.clone(),
            book: self.book.clone(),
            fonts: self.fonts.clone(),
            root: self.root.clone(),
            main,
        };
        let warned = typst::compile::<PagedDocument>(&world);
        let document = warned
            .output
            .map_err(|errs| format_diagnostics("compile", &errs))?;
        let bytes = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
            .map_err(|errs| format_diagnostics("pdf export", &errs))?;
        Ok(bytes)
    }
}

fn format_diagnostics(stage: &str, errs: &[SourceDiagnostic]) -> anyhow::Error {
    let mut s = String::new();
    for e in errs {
        if !s.is_empty() {
            s.push('\n');
        }
        s.push_str(&e.message);
        for h in &e.hints {
            s.push_str("\n  hint: ");
            s.push_str(h);
        }
    }
    anyhow::anyhow!("typst {stage}: {s}")
}

struct PdfWorld {
    library: Arc<LazyHash<Library>>,
    book: Arc<LazyHash<FontBook>>,
    fonts: Arc<Vec<FontSlot>>,
    root: PathBuf,
    main: Source,
}

impl World for PdfWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        let path = self.resolve(id)?;
        let text =
            std::fs::read_to_string(&path).map_err(|err| FileError::from_io(err, &path))?;
        Ok(Source::new(id, text))
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve(id)?;
        let bytes = std::fs::read(&path).map_err(|err| FileError::from_io(err, &path))?;
        Ok(Bytes::new(bytes))
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index)?.get()
    }
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        let now = Local::now();
        Datetime::from_ymd(now.year(), now.month() as u8, now.day() as u8)
    }
}

impl PdfWorld {
    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        if id.package().is_some() {
            return Err(FileError::Other(Some(
                "remote packages not supported".into(),
            )));
        }
        id.vpath()
            .resolve(&self.root)
            .ok_or(FileError::AccessDenied)
            .and_then(|p| {
                if path_within(&p, &self.root) {
                    Ok(p)
                } else {
                    Err(FileError::AccessDenied)
                }
            })
    }
}

fn path_within(path: &Path, root: &Path) -> bool {
    let canon = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let canon_root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };
    canon.starts_with(canon_root)
}

/// Wrap a post's pre-rendered Typst body in the `blog_post.typ` template call.
pub fn build_source(post: &Post) -> String {
    let mut s = String::with_capacity(post.body_typst.len() + 512);
    s.push_str("#import \"/templates/blog_post.typ\": render\n");
    s.push_str("#render(\n");
    writeln!(s, "  title: \"{}\",", str_lit(&post.title)).ok();
    writeln!(s, "  date: \"{}\",", str_lit(&post.date)).ok();
    writeln!(s, "  read_time: {},", post.read_time).ok();
    s.push_str("  tags: (");
    for (i, t) in post.tags.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        write!(s, "\"{}\"", str_lit(t)).ok();
    }
    if post.tags.len() == 1 {
        s.push(',');
    }
    s.push_str("),\n");
    writeln!(s, "  description: \"{}\",", str_lit(&post.description)).ok();
    if post.cover_image.is_empty() {
        s.push_str("  cover_image: none,\n");
    } else {
        writeln!(
            s,
            "  cover_image: \"/content/images/{}\",",
            str_lit(&post.cover_image)
        )
        .ok();
    }
    s.push_str("  body: [\n");
    s.push_str(&post.body_typst);
    s.push_str("\n  ],\n)\n");
    s
}

/// Convert markdown source into Typst markup body (used as `body` argument to
/// `blog_post.typ::render`). No off-the-shelf md→Typst converter exists, so we
/// walk comrak's AST and emit Typst by hand.
pub fn typst_from_markdown(md: &str) -> String {
    let arena = Arena::new();
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.render.unsafe_ = true;
    let root = comrak::parse_document(&arena, md, &opts);
    let mut out = String::with_capacity(md.len() * 2);
    render_block(root, &mut out);
    out
}

fn render_block<'a>(node: &'a AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Document => {
            for child in node.children() {
                render_block(child, out);
            }
        }
        NodeValue::Paragraph => {
            if is_paragraph_only_image(node) {
                if let Some(child) = node.children().next() {
                    if let NodeValue::Image(l) = &child.data.borrow().value {
                        emit_block_image(l, child, out);
                    }
                }
            } else {
                for child in node.children() {
                    render_inline(child, out);
                }
                out.push_str("\n\n");
            }
        }
        NodeValue::Heading(h) => {
            for _ in 0..h.level {
                out.push('=');
            }
            out.push(' ');
            for child in node.children() {
                render_inline(child, out);
            }
            out.push_str("\n\n");
        }
        NodeValue::List(l) => {
            let ordered = matches!(l.list_type, comrak::nodes::ListType::Ordered);
            for child in node.children() {
                emit_list_item(child, out, ordered);
            }
            out.push('\n');
        }
        NodeValue::Item(_) => {} // handled by parent List
        NodeValue::BlockQuote => {
            out.push_str("#quote(block: true)[\n");
            for child in node.children() {
                render_block(child, out);
            }
            out.push_str("]\n\n");
        }
        NodeValue::ThematicBreak => {
            out.push_str("#align(center)[#line(length: 30%, stroke: 0.5pt + gray)]\n\n");
        }
        NodeValue::CodeBlock(c) => {
            let lang = c.info.split_whitespace().next().unwrap_or("");
            out.push_str("#raw(block: true");
            if !lang.is_empty() {
                out.push_str(", lang: \"");
                out.push_str(&str_lit(lang));
                out.push('"');
            }
            out.push_str(", \"");
            out.push_str(&str_lit(&c.literal));
            out.push_str("\")\n\n");
        }
        NodeValue::HtmlBlock(_) => {} // skip raw HTML in PDF output
        NodeValue::Table(t) => emit_table(node, &t.alignments, out),
        _ => {
            for child in node.children() {
                render_block(child, out);
            }
        }
    }
}

fn render_inline<'a>(node: &'a AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(t) => out.push_str(&escape_markup(t)),
        NodeValue::SoftBreak => out.push(' '),
        NodeValue::LineBreak => out.push_str(" \\\n"),
        NodeValue::Code(c) => {
            out.push_str("#raw(\"");
            out.push_str(&str_lit(&c.literal));
            out.push_str("\")");
        }
        NodeValue::HtmlInline(_) => {} // drop raw HTML inline in PDF
        NodeValue::Emph => wrap_inline(node, "#emph[", "]", out),
        NodeValue::Strong => wrap_inline(node, "#strong[", "]", out),
        NodeValue::Strikethrough => wrap_inline(node, "#strike[", "]", out),
        NodeValue::Link(l) => {
            out.push_str("#link(\"");
            out.push_str(&str_lit(&l.url));
            out.push_str("\")[");
            for child in node.children() {
                render_inline(child, out);
            }
            out.push(']');
        }
        NodeValue::Image(l) => {
            out.push_str("#image(\"/content/images/");
            out.push_str(&str_lit(strip_images_prefix(&l.url)));
            out.push_str("\")");
        }
        _ => {
            for child in node.children() {
                render_inline(child, out);
            }
        }
    }
}

fn wrap_inline<'a>(node: &'a AstNode<'a>, open: &str, close: &str, out: &mut String) {
    out.push_str(open);
    for child in node.children() {
        render_inline(child, out);
    }
    out.push_str(close);
}

fn emit_list_item<'a>(item: &'a AstNode<'a>, out: &mut String, ordered: bool) {
    out.push_str(if ordered { "+ " } else { "- " });
    for child in item.children() {
        match &child.data.borrow().value {
            NodeValue::Paragraph => {
                for c in child.children() {
                    render_inline(c, out);
                }
            }
            NodeValue::List(l) => {
                let nested_ordered = matches!(l.list_type, comrak::nodes::ListType::Ordered);
                out.push('\n');
                for grandchild in child.children() {
                    out.push_str("  ");
                    emit_list_item(grandchild, out, nested_ordered);
                }
            }
            _ => render_block(child, out),
        }
    }
    out.push('\n');
}

fn emit_block_image<'a>(l: &comrak::nodes::NodeLink, node: &'a AstNode<'a>, out: &mut String) {
    let url = strip_images_prefix(&l.url);
    let mut alt = String::new();
    for child in node.children() {
        collect_text(child, &mut alt);
    }
    out.push_str("#align(center)[#image(\"/content/images/");
    out.push_str(&str_lit(url));
    out.push_str("\", width: 100%");
    if !alt.is_empty() {
        out.push_str(", alt: \"");
        out.push_str(&str_lit(&alt));
        out.push('"');
    }
    out.push_str(")]\n\n");
}

fn emit_table<'a>(node: &'a AstNode<'a>, alignments: &[TableAlignment], out: &mut String) {
    let mut rows = node.children().peekable();
    let columns = match rows.peek() {
        Some(first) => first.children().count(),
        None => return,
    };
    if columns == 0 {
        return;
    }
    out.push_str("#table(columns: ");
    out.push_str(&columns.to_string());
    out.push_str(", align: (");
    for i in 0..columns {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(match alignments.get(i).copied() {
            Some(TableAlignment::Right) => "right",
            Some(TableAlignment::Center) => "center",
            _ => "left",
        });
    }
    out.push_str("),\n");
    for row in rows {
        for cell in row.children() {
            out.push_str("  [");
            for child in cell.children() {
                render_inline(child, out);
            }
            out.push_str("],\n");
        }
    }
    out.push_str(")\n\n");
}

fn collect_text<'a>(node: &'a AstNode<'a>, buf: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(t) => buf.push_str(t),
        NodeValue::Code(c) => buf.push_str(&c.literal),
        _ => {
            for child in node.children() {
                collect_text(child, buf);
            }
        }
    }
}

fn is_paragraph_only_image<'a>(para: &'a AstNode<'a>) -> bool {
    let mut iter = para.children();
    let first = iter.next();
    if iter.next().is_some() {
        return false;
    }
    match first {
        Some(child) => matches!(child.data.borrow().value, NodeValue::Image(_)),
        None => false,
    }
}

fn strip_images_prefix(url: &str) -> &str {
    url.strip_prefix("images/").unwrap_or(url)
}

/// Backslash-escape characters with meaning in Typst markup mode.
fn escape_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | '[' | ']' | '*' | '_' | '`' | '#' | '$' | '<' | '@' | '~' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Escape a string for use inside a `"..."` Typst string literal.
fn str_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}
