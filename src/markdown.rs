use comrak::{
    nodes::{AstNode, NodeValue, TableAlignment},
    Arena, ComrakOptions,
};
use std::cell::RefCell;
use std::fmt::Write;

fn options() -> ComrakOptions {
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.render.unsafe_ = true;
    opts
}

pub fn render_blog(md: &str) -> String {
    let arena = Arena::new();
    let opts = options();
    let root = comrak::parse_document(&arena, md, &opts);
    let mut out = String::with_capacity(md.len() * 2);
    render_node(root, &mut out, &opts);
    out
}

pub fn render_typst(md: &str) -> String {
    let arena = Arena::new();
    let opts = options();
    let root = comrak::parse_document(&arena, md, &opts);
    let mut out = String::with_capacity(md.len() * 2);
    render_block_typst(root, &mut out);
    out
}

fn render_node<'a>(node: &'a AstNode<'a>, out: &mut String, opts: &ComrakOptions) {
    let value = &node.data.borrow().value;
    match value {
        NodeValue::Document => {
            for child in node.children() {
                render_node(child, out, opts);
            }
        }
        NodeValue::Paragraph => {
            // Handle the BlogRenderer paragraph -> block-image short-circuit:
            // if the paragraph contains exactly one image, render it as block-image
            // without wrapping in block-rich-text.
            let only_image = is_paragraph_only_image(node);
            if only_image {
                for child in node.children() {
                    render_inline(child, out, opts);
                }
            } else {
                out.push_str("<div class=\"block-rich-text\"><p>");
                for child in node.children() {
                    render_inline(child, out, opts);
                }
                out.push_str("</p></div>\n");
            }
        }
        NodeValue::Heading(h) => {
            let level = h.level;
            write!(out, "<div class=\"block-rich-text\"><h{level}>").ok();
            for child in node.children() {
                render_inline(child, out, opts);
            }
            write!(out, "</h{level}></div>\n").ok();
        }
        NodeValue::List(_) => {
            let ordered = matches!(
                value,
                NodeValue::List(l) if l.list_type == comrak::nodes::ListType::Ordered
            );
            let tag = if ordered { "ol" } else { "ul" };
            write!(out, "<div class=\"block-rich-text\"><{tag}>").ok();
            for child in node.children() {
                render_node(child, out, opts);
            }
            write!(out, "</{tag}></div>\n").ok();
        }
        NodeValue::Item(_) => {
            out.push_str("<li>");
            for child in node.children() {
                // Inside list items, render children directly without re-wrapping
                // (paragraph children become inline text).
                match &child.data.borrow().value {
                    NodeValue::Paragraph => {
                        for c in child.children() {
                            render_inline(c, out, opts);
                        }
                    }
                    _ => render_node(child, out, opts),
                }
            }
            out.push_str("</li>");
        }
        NodeValue::BlockQuote => {
            out.push_str("<div class=\"block-rich-text\"><blockquote>");
            for child in node.children() {
                render_node(child, out, opts);
            }
            out.push_str("</blockquote></div>\n");
        }
        NodeValue::ThematicBreak => {
            out.push_str("<div class=\"block-rich-text\"><hr></div>\n");
        }
        NodeValue::CodeBlock(c) => {
            let mut lang = c.info.split_whitespace().next().unwrap_or("").to_string();
            if lang == "html" {
                lang = "htmlmixed".to_string();
            }
            let escaped = html_escape(&c.literal);
            write!(
                out,
                "<div class=\"block-code\"><textarea data-language=\"{lang}\">{escaped}</textarea></div>\n"
            )
            .ok();
        }
        NodeValue::HtmlBlock(h) => {
            out.push_str(&h.literal);
        }
        NodeValue::Table(t) => {
            let alignments = &t.alignments;
            out.push_str(
                "<div class=\"block-rich-text\"><div class=\"table-responsive\"><table class=\"table\">",
            );
            let mut rows = node.children();
            if let Some(header) = rows.next() {
                out.push_str("<thead><tr>");
                for (i, cell) in header.children().enumerate() {
                    let style = align_style(alignments.get(i).copied());
                    write!(out, "<th{style}>").ok();
                    for child in cell.children() {
                        render_inline(child, out, opts);
                    }
                    out.push_str("</th>");
                }
                out.push_str("</tr></thead>");
            }
            out.push_str("<tbody>");
            for row in rows {
                out.push_str("<tr>");
                for (i, cell) in row.children().enumerate() {
                    let style = align_style(alignments.get(i).copied());
                    write!(out, "<td{style}>").ok();
                    for child in cell.children() {
                        render_inline(child, out, opts);
                    }
                    out.push_str("</td>");
                }
                out.push_str("</tr>");
            }
            out.push_str("</tbody></table></div></div>\n");
        }
        _ => {
            // Fallback: emit raw HTML for unhandled block types via comrak
            let buf = RefCell::new(String::new());
            // For simplicity, just iterate children
            for child in node.children() {
                render_node(child, out, opts);
            }
            drop(buf);
        }
    }
}

fn align_style(a: Option<TableAlignment>) -> &'static str {
    match a {
        Some(TableAlignment::Left) => " style=\"text-align:left\"",
        Some(TableAlignment::Right) => " style=\"text-align:right\"",
        Some(TableAlignment::Center) => " style=\"text-align:center\"",
        _ => "",
    }
}

fn is_paragraph_only_image<'a>(para: &'a AstNode<'a>) -> bool {
    let mut iter = para.children();
    let first = iter.next();
    let second = iter.next();
    if second.is_some() {
        return false;
    }
    match first {
        Some(child) => matches!(child.data.borrow().value, NodeValue::Image(_)),
        None => false,
    }
}

fn render_inline<'a>(node: &'a AstNode<'a>, out: &mut String, opts: &ComrakOptions) {
    let value = &node.data.borrow().value;
    match value {
        NodeValue::Text(t) => out.push_str(&html_escape(t)),
        NodeValue::SoftBreak => out.push('\n'),
        NodeValue::LineBreak => out.push_str("<br>\n"),
        NodeValue::Code(c) => {
            write!(out, "<code>{}</code>", html_escape(&c.literal)).ok();
        }
        NodeValue::HtmlInline(s) => out.push_str(s),
        NodeValue::Emph => {
            out.push_str("<em>");
            for child in node.children() {
                render_inline(child, out, opts);
            }
            out.push_str("</em>");
        }
        NodeValue::Strong => {
            out.push_str("<strong>");
            for child in node.children() {
                render_inline(child, out, opts);
            }
            out.push_str("</strong>");
        }
        NodeValue::Strikethrough => {
            out.push_str("<del>");
            for child in node.children() {
                render_inline(child, out, opts);
            }
            out.push_str("</del>");
        }
        NodeValue::Link(l) => {
            write!(out, "<a href=\"{}\"", html_escape(&l.url)).ok();
            if !l.title.is_empty() {
                write!(out, " title=\"{}\"", html_escape(&l.title)).ok();
            }
            out.push_str(">");
            for child in node.children() {
                render_inline(child, out, opts);
            }
            out.push_str("</a>");
        }
        NodeValue::Image(l) => {
            // BlogRenderer.image: strip "images/" prefix, wrap in block-image div
            let mut url = l.url.clone();
            if let Some(stripped) = url.strip_prefix("images/") {
                url = stripped.to_string();
            }
            // The "alt text" in comrak is the children rendered as plain text
            let mut alt = String::new();
            for child in node.children() {
                collect_text(child, &mut alt);
            }
            write!(
                out,
                "<div class=\"block-image\"><img src=\"/content/images/{}\" class=\"rounded\" alt=\"{}\"></div>\n",
                html_escape(&url),
                html_escape(&alt),
            )
            .ok();
        }
        _ => {
            // Fallback: just render children inline
            for child in node.children() {
                render_inline(child, out, opts);
            }
        }
    }
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

// Match Mistune's escape: & < > " (no apostrophe).
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

// ----- Typst conversion (used for PDF export) -----

fn render_block_typst<'a>(node: &'a AstNode<'a>, out: &mut String) {
    let value = &node.data.borrow().value;
    match value {
        NodeValue::Document => {
            for child in node.children() {
                render_block_typst(child, out);
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
                    render_inline_typst(child, out);
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
                render_inline_typst(child, out);
            }
            out.push_str("\n\n");
        }
        NodeValue::List(l) => {
            let ordered = matches!(l.list_type, comrak::nodes::ListType::Ordered);
            for child in node.children() {
                emit_list_item_typst(child, out, ordered);
            }
            out.push('\n');
        }
        NodeValue::Item(_) => {
            // Handled by parent List
        }
        NodeValue::BlockQuote => {
            out.push_str("#quote(block: true)[\n");
            for child in node.children() {
                render_block_typst(child, out);
            }
            out.push_str("]\n\n");
        }
        NodeValue::ThematicBreak => {
            out.push_str(
                "#align(center)[#line(length: 30%, stroke: 0.5pt + gray)]\n\n",
            );
        }
        NodeValue::CodeBlock(c) => {
            let lang = c.info.split_whitespace().next().unwrap_or("");
            out.push_str("#raw(block: true");
            if !lang.is_empty() {
                out.push_str(", lang: \"");
                out.push_str(&typst_str_lit(lang));
                out.push('"');
            }
            out.push_str(", \"");
            out.push_str(&typst_str_lit(&c.literal));
            out.push_str("\")\n\n");
        }
        NodeValue::HtmlBlock(_) => {
            // Skip raw HTML in PDF output.
        }
        NodeValue::Table(t) => {
            emit_table_typst(node, &t.alignments, out);
        }
        _ => {
            for child in node.children() {
                render_block_typst(child, out);
            }
        }
    }
}

fn emit_list_item_typst<'a>(item: &'a AstNode<'a>, out: &mut String, ordered: bool) {
    out.push_str(if ordered { "+ " } else { "- " });
    for child in item.children() {
        match &child.data.borrow().value {
            NodeValue::Paragraph => {
                for c in child.children() {
                    render_inline_typst(c, out);
                }
            }
            NodeValue::List(l) => {
                let nested_ordered =
                    matches!(l.list_type, comrak::nodes::ListType::Ordered);
                out.push('\n');
                for grandchild in child.children() {
                    out.push_str("  ");
                    emit_list_item_typst(grandchild, out, nested_ordered);
                }
            }
            _ => render_block_typst(child, out),
        }
    }
    out.push('\n');
}

fn emit_block_image<'a>(
    l: &comrak::nodes::NodeLink,
    node: &'a AstNode<'a>,
    out: &mut String,
) {
    let url = strip_images_prefix(&l.url);
    let mut alt = String::new();
    for child in node.children() {
        collect_text(child, &mut alt);
    }
    out.push_str("#align(center)[#image(\"/content/images/");
    out.push_str(&typst_str_lit(&url));
    out.push_str("\", width: 100%");
    if !alt.is_empty() {
        out.push_str(", alt: \"");
        out.push_str(&typst_str_lit(&alt));
        out.push('"');
    }
    out.push_str(")]\n\n");
}

fn emit_table_typst<'a>(
    node: &'a AstNode<'a>,
    alignments: &[TableAlignment],
    out: &mut String,
) {
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
    for (i, _) in (0..columns).enumerate() {
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
                render_inline_typst(child, out);
            }
            out.push_str("],\n");
        }
    }
    out.push_str(")\n\n");
}

fn render_inline_typst<'a>(node: &'a AstNode<'a>, out: &mut String) {
    let value = &node.data.borrow().value;
    match value {
        NodeValue::Text(t) => out.push_str(&typst_escape(t)),
        NodeValue::SoftBreak => out.push(' '),
        NodeValue::LineBreak => out.push_str(" \\\n"),
        NodeValue::Code(c) => {
            out.push_str("#raw(\"");
            out.push_str(&typst_str_lit(&c.literal));
            out.push_str("\")");
        }
        NodeValue::HtmlInline(_) => {
            // Drop raw HTML inline in PDF output.
        }
        NodeValue::Emph => {
            out.push_str("#emph[");
            for child in node.children() {
                render_inline_typst(child, out);
            }
            out.push(']');
        }
        NodeValue::Strong => {
            out.push_str("#strong[");
            for child in node.children() {
                render_inline_typst(child, out);
            }
            out.push(']');
        }
        NodeValue::Strikethrough => {
            out.push_str("#strike[");
            for child in node.children() {
                render_inline_typst(child, out);
            }
            out.push(']');
        }
        NodeValue::Link(l) => {
            out.push_str("#link(\"");
            out.push_str(&typst_str_lit(&l.url));
            out.push_str("\")[");
            for child in node.children() {
                render_inline_typst(child, out);
            }
            out.push(']');
        }
        NodeValue::Image(l) => {
            let url = strip_images_prefix(&l.url);
            out.push_str("#image(\"/content/images/");
            out.push_str(&typst_str_lit(&url));
            out.push_str("\")");
        }
        _ => {
            for child in node.children() {
                render_inline_typst(child, out);
            }
        }
    }
}

fn strip_images_prefix(url: &str) -> String {
    url.strip_prefix("images/").unwrap_or(url).to_string()
}

/// Backslash-escape characters that have meaning in Typst markup mode.
fn typst_escape(s: &str) -> String {
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
pub fn typst_str_lit(s: &str) -> String {
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
