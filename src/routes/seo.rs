use axum::{
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::{context, Value};
use std::collections::HashMap;

use crate::app::AppState;
use crate::error::AppError;
use crate::posts;
use crate::render::build_request;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/og/{slug_svg}", get(og_image))
        .route("/favicon.ico", get(favicon))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
}

fn wrap_title(title: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in title.split_whitespace() {
        if !current.is_empty() && current.len() + word.len() + 1 > max_chars {
            lines.push(current);
            current = word.to_string();
        } else if current.is_empty() {
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.into_iter().take(max_lines).collect()
}

async fn og_image(
    State(state): State<AppState>,
    AxumPath(slug_svg): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let request = build_request(&uri, &headers);
    let slug = slug_svg
        .strip_suffix(".svg")
        .unwrap_or(&slug_svg)
        .to_string();
    let (title, tags) = match state.posts_by_slug.get(&slug).copied() {
        Some(idx) => (state.posts[idx].title.clone(), state.posts[idx].tags.clone()),
        None => ("Isaac Bythewood's Blog".to_string(), Vec::new()),
    };
    let title_lines = wrap_title(&title, 35, 3);
    let tmpl = state.env.get_template("og.svg")?;
    let body = tmpl.render(context! { title_lines, tags, request => &request })?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "image/svg+xml".parse().unwrap());
    Ok((StatusCode::OK, h, body).into_response())
}

async fn favicon(State(state): State<AppState>) -> Result<Response, AppError> {
    let tmpl = state.env.get_template("favicon.svg")?;
    let body = tmpl.render(Value::UNDEFINED)?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "image/svg+xml".parse().unwrap());
    Ok((StatusCode::OK, h, body).into_response())
}

async fn robots(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let request = build_request(&uri, &headers);
    let tmpl = state.env.get_template("robots.txt")?;
    let body = tmpl.render(context! { request => &request })?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
    Ok((StatusCode::OK, h, body).into_response())
}

async fn sitemap(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let request = build_request(&uri, &headers);
    let published = posts::published(&state.posts);
    let tags = posts::collect_tags(&published);
    let years = posts::collect_years(&published);
    let mut tag_lastmod: HashMap<String, String> = HashMap::new();
    let mut year_lastmod: HashMap<String, String> = HashMap::new();
    for p in &published {
        for t in &p.tags {
            tag_lastmod
                .entry(t.clone())
                .and_modify(|cur| {
                    if p.date > *cur {
                        *cur = p.date.clone();
                    }
                })
                .or_insert_with(|| p.date.clone());
        }
        if p.date.len() >= 4 {
            let y = p.date[..4].to_string();
            year_lastmod
                .entry(y)
                .and_modify(|cur| {
                    if p.date > *cur {
                        *cur = p.date.clone();
                    }
                })
                .or_insert_with(|| p.date.clone());
        }
    }
    let tmpl = state.env.get_template("sitemap.xml")?;
    let body = tmpl.render(context! {
        posts => published, tags, years, tag_lastmod, year_lastmod, request => &request,
    })?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "application/xml".parse().unwrap());
    Ok((StatusCode::OK, h, body).into_response())
}
