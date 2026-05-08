mod markdown;
mod pdf;
mod posts;
mod templates;

use axum::{
    body::Body,
    extract::{Path as AxumPath, Query, Request, State},
    http::{header, HeaderMap, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::{Datelike, Local};
use minijinja::{context, Environment, Value};
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use posts::Post;
use templates::RequestCtx;

#[derive(Clone)]
struct AppState {
    env: Arc<Environment<'static>>,
    posts: Arc<Vec<Post>>,
    posts_by_slug: Arc<HashMap<String, usize>>,
    content_dir: PathBuf,
    pdf_renderer: Arc<pdf::PdfRenderer>,
}

#[tokio::main]
async fn main() {
    let project_root: PathBuf = std::env::var("BLOG_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let templates_dir = project_root.join("templates");
    let dist_dir = project_root.join("dist");
    let content_dir = project_root.join("content");
    let manifest_path = dist_dir.join(".vite/manifest.json");

    let env = templates::build_env(&templates_dir, &manifest_path);
    let posts = posts::load_posts(&content_dir);
    let posts_by_slug: HashMap<String, usize> = posts
        .iter()
        .enumerate()
        .map(|(i, p)| (p.slug.clone(), i))
        .collect();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);

    let pdf_renderer = Arc::new(pdf::PdfRenderer::new(project_root.clone()));

    let state = AppState {
        env: Arc::new(env),
        posts: Arc::new(posts),
        posts_by_slug: Arc::new(posts_by_slug),
        content_dir: content_dir.clone(),
        pdf_renderer,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/blog/", get(blog_index))
        .route("/posts/{slug}/", get(blog_post))
        .route("/posts/{slug}/pdf/", get(blog_post_pdf))
        .route("/posts/{slug}/md/", get(blog_post_md))
        .route("/blog/{slug}/", get(blog_post_redirect))
        .route("/blog/{slug}/pdf/", get(blog_post_pdf_redirect))
        .route("/blog/{slug}/md/", get(blog_post_md_redirect))
        .route("/blog/tag/{tag}/", get(blog_tag))
        .route("/blog/year/{year}/", get(blog_year))
        .route("/search/", get(search_page))
        .route("/search/live/", get(search_live))
        .route("/og/{slug_svg}", get(og_image))
        .route("/favicon.ico", get(favicon))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
        .nest_service(
            "/static",
            tower::ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("public, max-age=31536000"),
                ))
                .service(ServeDir::new(&dist_dir)),
        )
        .nest_service(
            "/content/images",
            tower::ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("public, max-age=31536000"),
                ))
                .service(ServeDir::new(content_dir.join("images"))),
        )
        .fallback(not_found)
        .layer(middleware::from_fn(log_requests))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    eprintln!("blog listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

fn today() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

async fn log_requests(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req
        .uri()
        .path_and_query()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let start = Instant::now();
    let response = next.run(req).await;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let status = response.status().as_u16();
    let now = Local::now().format("%H:%M:%S");
    let color = match status {
        200..=299 => "\x1b[32m", // green
        300..=399 => "\x1b[36m", // cyan
        400..=499 => "\x1b[33m", // yellow
        _ => "\x1b[31m",          // red
    };
    eprintln!(
        "{now} {method:<5} {color}{status}\x1b[0m {elapsed_ms:>7.2}ms  {path}"
    );
    response
}

fn published_posts(state: &AppState) -> Vec<Post> {
    let today = today();
    state
        .posts
        .iter()
        .filter(|p| p.publish_date.as_str() <= today.as_str())
        .cloned()
        .collect()
}

fn collect_tags(posts: &[Post]) -> Vec<TagEntry> {
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

fn collect_years(posts: &[Post]) -> Vec<String> {
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

fn related(post: &Post, posts: &[Post], count: usize) -> Vec<Post> {
    if post.tags.is_empty() {
        return posts.iter().take(count).cloned().collect();
    }
    let post_tags: std::collections::HashSet<&String> = post.tags.iter().collect();
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
        let have: std::collections::HashSet<String> =
            out.iter().map(|p| p.slug.clone()).collect();
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

#[derive(Debug, Clone, serde::Serialize)]
struct TagEntry {
    name: String,
    slug: String,
    count: usize,
    url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct Crumb {
    title: String,
    url: String,
}

fn build_request(uri: &Uri, headers: &HeaderMap) -> RequestCtx {
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

fn render_html(
    state: &AppState,
    template: &str,
    extra: minijinja::Value,
    request: &RequestCtx,
) -> Result<Html<String>, AppError> {
    let posts = published_posts(state);
    let nav_items = collect_tags(&posts);
    let now = NowCtx { year: Local::now().year() };
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

#[derive(Debug, serde::Serialize)]
struct NowCtx {
    year: i32,
}

struct AppError(StatusCode, String);

impl AppError {
    fn not_found() -> Self {
        AppError(StatusCode::NOT_FOUND, "not found".to_string())
    }
}

impl<E: std::fmt::Display> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, format!("internal error: {e}"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

async fn index(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let posts = published_posts(&state);
    let latest_post = posts.first().cloned();
    let rest: Vec<Post> = match &latest_post {
        Some(latest) => posts.iter().filter(|p| p.slug != latest.slug).cloned().collect(),
        None => Vec::new(),
    };
    let mut rng = rand::thread_rng();
    let mut shuffled = rest.clone();
    shuffled.shuffle(&mut rng);
    let random_blog_posts: Vec<Post> = shuffled.into_iter().take(3).collect();

    let page = context! {
        title => "Isaac Bythewood's Blog",
        slug => "home",
        description => "Writing about webdev, infrastructure, security, and tooling by Isaac Bythewood, a Senior Solutions Architect in Elkin, NC.",
    };

    render_html(&state, "home.html", context! { page, latest_post, random_blog_posts }, &request)
}

async fn blog_index(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let posts = published_posts(&state);
    let tags = collect_tags(&posts);
    let years = collect_years(&posts);
    let breadcrumbs = vec![Crumb { title: "Home".into(), url: "/".into() }];
    let page = context! {
        title => "Blog",
        slug => "blog",
        description => "Posts on webdev, coding, security, and sysadmin by Isaac Bythewood.",
    };
    render_html(
        &state,
        "blog_index.html",
        context! { page, blog_posts => posts, tags, years, breadcrumbs },
        &request,
    )
}

async fn blog_post(
    State(state): State<AppState>,
    AxumPath(slug): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let idx = state.posts_by_slug.get(&slug).copied().ok_or_else(AppError::not_found)?;
    let post = state.posts[idx].clone();
    if post.publish_date.as_str() > today().as_str() {
        return Err(AppError::not_found());
    }
    let posts = published_posts(&state);
    let related_posts = related(&post, &posts, 3);
    let breadcrumbs = vec![
        Crumb { title: "Home".into(), url: "/".into() },
        Crumb { title: "Blog".into(), url: "/blog/".into() },
    ];
    render_html(
        &state,
        "blog_post.html",
        context! { page => &post, post => &post, related_posts, breadcrumbs },
        &request,
    )
}

async fn blog_post_pdf(
    State(state): State<AppState>,
    AxumPath(slug): AxumPath<String>,
) -> Result<Response, AppError> {
    let idx = state.posts_by_slug.get(&slug).copied().ok_or_else(AppError::not_found)?;
    let post = state.posts[idx].clone();
    if post.publish_date.as_str() > today().as_str() {
        return Err(AppError::not_found());
    }
    let source = build_typst_source(&post);
    let renderer = state.pdf_renderer.clone();
    let pdf = tokio::task::spawn_blocking(move || renderer.render(source))
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    h.insert(
        header::CONTENT_DISPOSITION,
        format!("filename=\"{}.pdf\"", post.slug).parse().unwrap(),
    );
    Ok((StatusCode::OK, h, Body::from(pdf)).into_response())
}

fn build_typst_source(post: &Post) -> String {
    use std::fmt::Write;
    let q = markdown::typst_str_lit;
    let mut s = String::with_capacity(post.body_typst.len() + 512);
    s.push_str("#import \"/templates/blog_post.typ\": render\n");
    s.push_str("#render(\n");
    writeln!(s, "  title: \"{}\",", q(&post.title)).ok();
    writeln!(s, "  date: \"{}\",", q(&post.date)).ok();
    writeln!(s, "  read_time: {},", post.read_time).ok();
    s.push_str("  tags: (");
    for (i, t) in post.tags.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        write!(s, "\"{}\"", q(t)).ok();
    }
    if post.tags.len() == 1 {
        s.push(',');
    }
    s.push_str("),\n");
    writeln!(s, "  description: \"{}\",", q(&post.description)).ok();
    if post.cover_image.is_empty() {
        s.push_str("  cover_image: none,\n");
    } else {
        writeln!(
            s,
            "  cover_image: \"/content/images/{}\",",
            q(&post.cover_image)
        )
        .ok();
    }
    s.push_str("  body: [\n");
    s.push_str(&post.body_typst);
    s.push_str("\n  ],\n)\n");
    s
}

async fn blog_post_md(
    State(state): State<AppState>,
    AxumPath(slug): AxumPath<String>,
) -> Result<Response, AppError> {
    let idx = state.posts_by_slug.get(&slug).copied().ok_or_else(AppError::not_found)?;
    let post = state.posts[idx].clone();
    if post.publish_date.as_str() > today().as_str() {
        return Err(AppError::not_found());
    }
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

async fn blog_post_redirect(AxumPath(slug): AxumPath<String>) -> Redirect {
    Redirect::permanent(&format!("/posts/{slug}/"))
}

async fn blog_post_pdf_redirect(AxumPath(slug): AxumPath<String>) -> Redirect {
    Redirect::permanent(&format!("/posts/{slug}/pdf/"))
}

async fn blog_post_md_redirect(AxumPath(slug): AxumPath<String>) -> Redirect {
    Redirect::permanent(&format!("/posts/{slug}/md/"))
}

async fn blog_tag(
    State(state): State<AppState>,
    AxumPath(tag): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let posts = published_posts(&state);
    let filtered: Vec<Post> = posts.iter().filter(|p| p.tags.contains(&tag)).cloned().collect();
    if filtered.is_empty() {
        return Err(AppError::not_found());
    }
    let extra_posts: Option<Vec<Post>> = if filtered.len() < 5 {
        Some(posts.iter().filter(|p| !p.tags.contains(&tag)).take(4).cloned().collect())
    } else {
        None
    };
    let tags = collect_tags(&posts);
    let years = collect_years(&posts);
    let active_tag = context! { name => &tag, slug => &tag };
    let page = context! {
        title => format!("Tag: {tag}"),
        slug => format!("tag-{tag}"),
        description => format!("Posts tagged {tag}"),
    };
    let breadcrumbs = vec![
        Crumb { title: "Home".into(), url: "/".into() },
        Crumb { title: "Blog".into(), url: "/blog/".into() },
    ];
    render_html(
        &state,
        "blog_index.html",
        context! { page, blog_posts => filtered, extra_posts, active_tag, tags, years, breadcrumbs },
        &request,
    )
}

async fn blog_year(
    State(state): State<AppState>,
    AxumPath(year): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let posts = published_posts(&state);
    let filtered: Vec<Post> = posts.iter().filter(|p| p.date.starts_with(&year)).cloned().collect();
    if filtered.is_empty() {
        return Err(AppError::not_found());
    }
    let extra_posts: Option<Vec<Post>> = if filtered.len() < 5 {
        Some(posts.iter().filter(|p| !p.date.starts_with(&year)).take(4).cloned().collect())
    } else {
        None
    };
    let tags = collect_tags(&posts);
    let years = collect_years(&posts);
    let page = context! {
        title => format!("Year: {year}"),
        slug => format!("year-{year}"),
        description => format!("Posts from {year}"),
    };
    let breadcrumbs = vec![
        Crumb { title: "Home".into(), url: "/".into() },
        Crumb { title: "Blog".into(), url: "/blog/".into() },
    ];
    render_html(
        &state,
        "blog_index.html",
        context! { page, blog_posts => filtered, extra_posts, active_year => &year, tags, years, breadcrumbs },
        &request,
    )
}

#[derive(Deserialize)]
struct SearchQuery {
    #[serde(default)]
    q: String,
}

async fn search_page(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let posts = published_posts(&state);
    let mut results: Vec<Post> = Vec::new();
    let mut random_posts: Option<Vec<Post>> = None;
    if !q.q.is_empty() {
        let ql = q.q.to_lowercase();
        for p in &posts {
            if p.title.to_lowercase().contains(&ql)
                || p.description.to_lowercase().contains(&ql)
                || p.tags.iter().any(|t| t.to_lowercase().contains(&ql))
            {
                results.push(p.clone());
            }
        }
    } else {
        let mut rng = rand::thread_rng();
        let mut shuffled = posts.clone();
        shuffled.shuffle(&mut rng);
        random_posts = Some(shuffled.into_iter().take(6).collect());
    }
    let breadcrumbs = vec![Crumb { title: "Home".into(), url: "/".into() }];
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

async fn search_live(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Response {
    let posts = published_posts(&state);
    let mut out = Vec::new();
    if !q.q.is_empty() {
        let ql = q.q.to_lowercase();
        for p in &posts {
            if p.title.to_lowercase().contains(&ql)
                || p.description.to_lowercase().contains(&ql)
                || p.tags.iter().any(|t| t.to_lowercase().contains(&ql))
            {
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

async fn og_image(
    State(state): State<AppState>,
    AxumPath(slug_svg): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let request = build_request(&uri, &headers);
    let slug = slug_svg.strip_suffix(".svg").unwrap_or(&slug_svg).to_string();
    let (title, tags) = match state.posts_by_slug.get(&slug).copied() {
        Some(idx) => (state.posts[idx].title.clone(), state.posts[idx].tags.clone()),
        None => ("Isaac Bythewood's Blog".to_string(), Vec::new()),
    };
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in title.split_whitespace() {
        if !current.is_empty() && current.len() + word.len() + 1 > 35 {
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
    let title_lines: Vec<String> = lines.into_iter().take(3).collect();
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
    let posts = published_posts(&state);
    let tags = collect_tags(&posts);
    let years = collect_years(&posts);
    let mut tag_lastmod: HashMap<String, String> = HashMap::new();
    let mut year_lastmod: HashMap<String, String> = HashMap::new();
    for p in &posts {
        for t in &p.tags {
            tag_lastmod
                .entry(t.clone())
                .and_modify(|cur| { if p.date > *cur { *cur = p.date.clone(); } })
                .or_insert_with(|| p.date.clone());
        }
        if p.date.len() >= 4 {
            let y = p.date[..4].to_string();
            year_lastmod
                .entry(y)
                .and_modify(|cur| { if p.date > *cur { *cur = p.date.clone(); } })
                .or_insert_with(|| p.date.clone());
        }
    }
    let tmpl = state.env.get_template("sitemap.xml")?;
    let body = tmpl.render(context! {
        posts, tags, years, tag_lastmod, year_lastmod, request => &request,
    })?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "application/xml".parse().unwrap());
    Ok((StatusCode::OK, h, body).into_response())
}

async fn not_found(
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
