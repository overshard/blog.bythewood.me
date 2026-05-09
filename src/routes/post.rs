use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::context;

use crate::app::AppState;
use crate::error::AppError;
use crate::pdf;
use crate::posts::{self, Post};
use crate::render::{build_request, render_html, Crumb};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/posts/{slug}/", get(show))
        .route("/posts/{slug}/pdf/", get(pdf_route))
        .route("/posts/{slug}/md/", get(markdown_route))
}

fn lookup(state: &AppState, slug: &str) -> Result<Post, AppError> {
    let idx = state
        .posts_by_slug
        .get(slug)
        .copied()
        .ok_or_else(AppError::not_found)?;
    let post = state.posts[idx].clone();
    if !posts::is_published(&post) {
        return Err(AppError::not_found());
    }
    Ok(post)
}

async fn show(
    State(state): State<AppState>,
    AxumPath(slug): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let post = lookup(&state, &slug)?;
    let published = posts::published(&state.posts);
    let related_posts = posts::related(&post, &published, 3);
    let breadcrumbs = vec![
        Crumb {
            title: "Home".into(),
            url: "/".into(),
        },
        Crumb {
            title: "Blog".into(),
            url: "/blog/".into(),
        },
    ];
    render_html(
        &state,
        "blog_post.html",
        context! { page => &post, post => &post, related_posts, breadcrumbs },
        &request,
    )
}

async fn pdf_route(
    State(state): State<AppState>,
    AxumPath(slug): AxumPath<String>,
) -> Result<Response, AppError> {
    let post = lookup(&state, &slug)?;
    let source = pdf::build_source(&post);
    let renderer = state.pdf_renderer.clone();
    let bytes = tokio::task::spawn_blocking(move || renderer.render(source))
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    h.insert(
        header::CONTENT_DISPOSITION,
        format!("filename=\"{}.pdf\"", post.slug).parse().unwrap(),
    );
    Ok((StatusCode::OK, h, Body::from(bytes)).into_response())
}

async fn markdown_route(
    State(state): State<AppState>,
    AxumPath(slug): AxumPath<String>,
) -> Result<Response, AppError> {
    let post = lookup(&state, &slug)?;
    let path = state.content_dir.join("posts").join(&post.filename);
    let bytes = tokio::fs::read(&path).await?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "text/markdown".parse().unwrap());
    h.insert(
        header::CONTENT_DISPOSITION,
        format!("filename=\"{}.md\"", post.slug).parse().unwrap(),
    );
    Ok((StatusCode::OK, h, Body::from(bytes)).into_response())
}
