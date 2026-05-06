use comrak::{
    nodes::{AstNode, NodeValue},
    Arena, ComrakOptions,
};
use std::cell::RefCell;
use std::fmt::Write;

fn options() -> ComrakOptions {
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    opts.render.unsafe_ = true;
    opts
}

pub fn render_blog(md: &str) -> String {
    let arena = Arena::new();
    let opts = options();
    let root = comrak::parse_document(&arena, md, &opts);
    let mut out = String::with_capacity(md.len() * 2);
    render_node(root, &mut out, &opts, false);
    out
}

pub fn render_pdf(md: &str) -> String {
    let arena = Arena::new();
    let opts = options();
    let root = comrak::parse_document(&arena, md, &opts);
    let mut out = String::with_capacity(md.len() * 2);
    render_node(root, &mut out, &opts, true);
    out
}

fn render_node<'a>(node: &'a AstNode<'a>, out: &mut String, opts: &ComrakOptions, pdf: bool) {
    let value = &node.data.borrow().value;
    match value {
        NodeValue::Document => {
            for child in node.children() {
                render_node(child, out, opts, pdf);
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
                render_node(child, out, opts, pdf);
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
                    _ => render_node(child, out, opts, pdf),
                }
            }
            out.push_str("</li>");
        }
        NodeValue::BlockQuote => {
            out.push_str("<div class=\"block-rich-text\"><blockquote>");
            for child in node.children() {
                render_node(child, out, opts, pdf);
            }
            out.push_str("</blockquote></div>\n");
        }
        NodeValue::ThematicBreak => {
            out.push_str("<div class=\"block-rich-text\"><hr></div>\n");
        }
        NodeValue::CodeBlock(c) => {
            let mut lang = c.info.split_whitespace().next().unwrap_or("").to_string();
            if !pdf && lang == "html" {
                lang = "htmlmixed".to_string();
            }
            let escaped = html_escape(&c.literal);
            if pdf {
                write!(
                    out,
                    "<div class=\"block-code\"><pre><code class=\"language-{lang}\">{escaped}</code></pre></div>\n"
                )
                .ok();
            } else {
                write!(
                    out,
                    "<div class=\"block-code\"><textarea data-language=\"{lang}\">{escaped}</textarea></div>\n"
                )
                .ok();
            }
        }
        NodeValue::HtmlBlock(h) => {
            out.push_str(&h.literal);
        }
        _ => {
            // Fallback: emit raw HTML for unhandled block types via comrak
            let buf = RefCell::new(String::new());
            // For simplicity, just iterate children
            for child in node.children() {
                render_node(child, out, opts, pdf);
            }
            drop(buf);
        }
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
