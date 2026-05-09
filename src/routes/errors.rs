use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use minijinja::context;

use crate::app::AppState;
use crate::render::{build_request, render_html};

pub async fn not_found(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let request = build_request(&uri, &headers);
    let page = context! { title => "404", description => "Page not found" };
    match render_html(&state, "404.html", context! { page }, &request) {
        Ok(html) => (StatusCode::NOT_FOUND, html).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}
