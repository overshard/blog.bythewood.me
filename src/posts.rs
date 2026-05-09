use chrono::Local;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::markdown;
use crate::pdf;

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
        let body_html = markdown::render(body);
        let body_typst = pdf::typst_from_markdown(body);
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

#[derive(Debug, Clone, Serialize)]
pub struct TagEntry {
    pub name: String,
    pub slug: String,
    pub count: usize,
    pub url: String,
}

pub fn today() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

pub fn is_published(post: &Post) -> bool {
    post.publish_date.as_str() <= today().as_str()
}

pub fn published(posts: &[Post]) -> Vec<Post> {
    posts.iter().filter(|p| is_published(p)).cloned().collect()
}

pub fn collect_tags(posts: &[Post]) -> Vec<TagEntry> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for p in posts {
        for t in &p.tags {
            *counts.entry(t.clone()).or_insert(0) += 1;
        }
    }
    let mut out: Vec<TagEntry> = counts
        .into_iter()
        .map(|(name, count)| TagEntry {
            url: format!("/blog/tag/{}/", urlencoding::encode(&name)),
            slug: name.clone(),
            name,
            count,
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

pub fn collect_years(posts: &[Post]) -> Vec<String> {
    let mut years: Vec<String> = posts
        .iter()
        .filter(|p| !p.date.is_empty())
        .map(|p| p.date[..4.min(p.date.len())].to_string())
        .collect();
    years.sort();
    years.dedup();
    years.reverse();
    years
}

pub fn related(post: &Post, posts: &[Post], count: usize) -> Vec<Post> {
    if post.tags.is_empty() {
        return posts.iter().take(count).cloned().collect();
    }
    let post_tags: HashSet<&String> = post.tags.iter().collect();
    let mut scored: Vec<(usize, &Post)> = posts
        .iter()
        .filter(|p| p.slug != post.slug)
        .map(|p| {
            let overlap = p.tags.iter().filter(|t| post_tags.contains(t)).count();
            (overlap, p)
        })
        .filter(|(o, _)| *o > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out: Vec<Post> = scored
        .into_iter()
        .take(count)
        .map(|(_, p)| p.clone())
        .collect();
    if out.len() < count {
        let have: HashSet<String> = out.iter().map(|p| p.slug.clone()).collect();
        for p in posts {
            if p.slug == post.slug || have.contains(&p.slug) {
                continue;
            }
            out.push(p.clone());
            if out.len() >= count {
                break;
            }
        }
    }
    out
}
