use axum::{http::header, middleware as axum_middleware, Router};
use minijinja::Environment;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::middleware::log_requests;
use crate::pdf::PdfRenderer;
use crate::posts::{self, Post};
use crate::routes;
use crate::templates;

#[derive(Clone)]
pub struct AppState {
    pub env: Arc<Environment<'static>>,
    pub posts: Arc<Vec<Post>>,
    pub posts_by_slug: Arc<HashMap<String, usize>>,
    pub content_dir: PathBuf,
    pub dist_dir: PathBuf,
    pub pdf_renderer: Arc<PdfRenderer>,
}

impl AppState {
    pub fn from_env() -> Self {
        let project_root: PathBuf = std::env::var("BLOG_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let templates_dir = project_root.join("templates");
        let dist_dir = project_root.join("dist");
        let content_dir = project_root.join("content");
        let manifest_path = dist_dir.join(".vite/manifest.json");

        let env = templates::build_env(&templates_dir, &manifest_path);
        let loaded = posts::load_posts(&content_dir);
        let posts_by_slug: HashMap<String, usize> = loaded
            .iter()
            .enumerate()
            .map(|(i, p)| (p.slug.clone(), i))
            .collect();
        let pdf_renderer = Arc::new(PdfRenderer::new(project_root));

        Self {
            env: Arc::new(env),
            posts: Arc::new(loaded),
            posts_by_slug: Arc::new(posts_by_slug),
            content_dir,
            dist_dir,
            pdf_renderer,
        }
    }
}

pub fn router(state: AppState) -> Router {
    let cache_static = || {
        SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("public, max-age=31536000"),
        )
    };
    let static_files = tower::ServiceBuilder::new()
        .layer(cache_static())
        .service(ServeDir::new(&state.dist_dir));
    let images = tower::ServiceBuilder::new()
        .layer(cache_static())
        .service(ServeDir::new(state.content_dir.join("images")));

    Router::new()
        .merge(routes::home::router())
        .merge(routes::blog::router())
        .merge(routes::post::router())
        .merge(routes::search::router())
        .merge(routes::seo::router())
        .nest_service("/static", static_files)
        .nest_service("/content/images", images)
        .fallback(routes::errors::not_found)
        .layer(axum_middleware::from_fn(log_requests))
        .with_state(state)
}
