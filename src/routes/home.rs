use axum::{
    extract::State,
    http::{HeaderMap, Uri},
    response::Html,
    routing::get,
    Router,
};
use minijinja::context;
use rand::seq::SliceRandom;

use crate::app::AppState;
use crate::error::AppError;
use crate::posts::{self, Post};
use crate::render::{build_request, render_html};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(index))
}

async fn index(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let published = posts::published(&state.posts);
    let latest_post = published.first().cloned();
    let rest: Vec<Post> = match &latest_post {
        Some(latest) => published
            .iter()
            .filter(|p| p.slug != latest.slug)
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    let mut rng = rand::thread_rng();
    let mut shuffled = rest;
    shuffled.shuffle(&mut rng);
    let random_blog_posts: Vec<Post> = shuffled.into_iter().take(3).collect();

    let page = context! {
        title => "Isaac Bythewood's Blog",
        slug => "home",
        description => "Writing about webdev, infrastructure, security, and tooling by Isaac Bythewood, a Senior Solutions Architect in Elkin, NC.",
    };

    render_html(
        &state,
        "home.html",
        context! { page, latest_post, random_blog_posts },
        &request,
    )
}
