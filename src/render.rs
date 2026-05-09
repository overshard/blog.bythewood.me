use axum::http::{header, HeaderMap, Uri};
use axum::response::Html;
use chrono::{Datelike, Local};
use minijinja::context;
use serde::Serialize;

use crate::app::AppState;
use crate::error::AppError;
use crate::posts::{self, collect_tags};
use crate::templates::RequestCtx;

#[derive(Debug, Clone, Serialize)]
pub struct Crumb {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
struct NowCtx {
    year: i32,
}

pub fn build_request(uri: &Uri, headers: &HeaderMap) -> RequestCtx {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let url_root = format!("{scheme}://{host}/");
    let path_and_query = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let url = format!("{scheme}://{host}{path_and_query}");
    let base_url = format!("{scheme}://{host}{}", uri.path());
    RequestCtx {
        url,
        url_root,
        base_url,
    }
}

pub fn render_html(
    state: &AppState,
    template: &str,
    extra: minijinja::Value,
    request: &RequestCtx,
) -> Result<Html<String>, AppError> {
    let published = posts::published(&state.posts);
    let nav_items = collect_tags(&published);
    let now = NowCtx {
        year: Local::now().year(),
    };
    let tmpl = state.env.get_template(template)?;
    let ctx = context! {
        nav_items => nav_items,
        now => now,
        debug => false,
        request => request,
        ..extra
    };
    let body = tmpl.render(ctx)?;
    Ok(Html(body))
}
