use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::markdown;

#[derive(Debug, Clone, Serialize)]
pub struct Post {
    pub filename: String,
    pub title: String,
    pub slug: String,
    pub date: String,
    pub publish_date: String,
    pub tags: Vec<String>,
    pub description: String,
    pub cover_image: String,
    pub body_html: String,
    pub body_typst: String,
    pub read_time: usize,
}

pub fn parse_frontmatter(text: &str) -> (HashMap<String, String>, &str) {
    let mut meta = HashMap::new();
    if !text.starts_with("---") {
        return (meta, text);
    }
    let after_first = &text[3..];
    let end_rel = match after_first.find("---") {
        Some(e) => e,
        None => return (meta, text),
    };
    let block = &after_first[..end_rel];
    let body_start = 3 + end_rel + 3;
    let body = text[body_start..].trim_start_matches(['\r', '\n', ' ', '\t']);
    for line in block.trim().lines() {
        if let Some((k, v)) = line.split_once(": ") {
            meta.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    (meta, body)
}

pub fn load_posts(content_dir: &PathBuf) -> Vec<Post> {
    let posts_dir = content_dir.join("posts");
    let mut posts = Vec::new();
    let entries = match fs::read_dir(&posts_dir) {
        Ok(e) => e,
        Err(_) => return posts,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let (meta, body) = parse_frontmatter(&text);
        let tags: Vec<String> = meta
            .get("tags")
            .map(|s| {
                s.split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let date = meta.get("date").cloned().unwrap_or_default();
        let publish_date = meta.get("publish_date").cloned().unwrap_or_else(|| date.clone());
        let body_html = markdown::render_blog(body);
        let body_typst = markdown::render_typst(body);
        let word_count = body.split_whitespace().count();
        let read_time = ((word_count as f64) / 200.0).ceil() as usize;
        let read_time = read_time.max(1);

        let slug = meta
            .get("slug")
            .cloned()
            .unwrap_or_else(|| filename.trim_end_matches(".md").to_string());

        posts.push(Post {
            filename,
            title: meta.get("title").cloned().unwrap_or_default(),
            slug,
            date,
            publish_date,
            tags,
            description: meta.get("description").cloned().unwrap_or_default(),
            cover_image: meta.get("cover_image").cloned().unwrap_or_default(),
            body_html,
            body_typst,
            read_time,
        });
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}
