use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::context;
use rand::seq::SliceRandom;
use serde::Deserialize;

use crate::app::AppState;
use crate::error::AppError;
use crate::posts::{self, Post};
use crate::render::{build_request, render_html, Crumb};

#[derive(Deserialize)]
struct SearchQuery {
    #[serde(default)]
    q: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search/", get(page))
        .route("/search/live/", get(live))
}

fn matches(post: &Post, needle_lower: &str) -> bool {
    post.title.to_lowercase().contains(needle_lower)
        || post.description.to_lowercase().contains(needle_lower)
        || post
            .tags
            .iter()
            .any(|t| t.to_lowercase().contains(needle_lower))
}

async fn page(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let published = posts::published(&state.posts);
    let mut results: Vec<Post> = Vec::new();
    let mut random_posts: Option<Vec<Post>> = None;
    if !q.q.is_empty() {
        let needle = q.q.to_lowercase();
        for p in &published {
            if matches(p, &needle) {
                results.push(p.clone());
            }
        }
    } else {
        let mut rng = rand::thread_rng();
        let mut shuffled = published.clone();
        shuffled.shuffle(&mut rng);
        random_posts = Some(shuffled.into_iter().take(6).collect());
    }
    let breadcrumbs = vec![Crumb {
        title: "Home".into(),
        url: "/".into(),
    }];
    let page = context! {
        title => "Search",
        slug => "search",
        description => "Search posts on webdev, coding, security, and sysadmin.",
    };
    render_html(
        &state,
        "search.html",
        context! { page, results, random_posts, q => &q.q, breadcrumbs },
        &request,
    )
}

async fn live(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> Response {
    let published = posts::published(&state.posts);
    let mut out = Vec::new();
    if !q.q.is_empty() {
        let needle = q.q.to_lowercase();
        for p in &published {
            if matches(p, &needle) {
                out.push(serde_json::json!({
                    "title": p.title,
                    "description": p.description,
                    "url": format!("/posts/{}/", p.slug),
                }));
                if out.len() >= 5 {
                    break;
                }
            }
        }
    }
    // Match Flask's jsonify: trailing newline.
    let body = serde_json::to_string(&out).unwrap_or_default() + "\n";
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    (StatusCode::OK, h, body).into_response()
}
