use axum::{
    extract::{Path as AxumPath, State},
    http::{HeaderMap, Uri},
    response::{Html, Redirect},
    routing::get,
    Router,
};
use minijinja::context;

use crate::app::AppState;
use crate::error::AppError;
use crate::posts::{self, Post};
use crate::render::{build_request, render_html, Crumb};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/blog/", get(index))
        .route("/blog/tag/{tag}/", get(by_tag))
        .route("/blog/year/{year}/", get(by_year))
        .route("/blog/{slug}/", get(post_redirect))
        .route("/blog/{slug}/pdf/", get(post_pdf_redirect))
        .route("/blog/{slug}/md/", get(post_md_redirect))
}

async fn index(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let published = posts::published(&state.posts);
    let tags = posts::collect_tags(&published);
    let years = posts::collect_years(&published);
    let breadcrumbs = vec![Crumb {
        title: "Home".into(),
        url: "/".into(),
    }];
    let page = context! {
        title => "Blog",
        slug => "blog",
        description => "Posts on webdev, coding, security, and sysadmin by Isaac Bythewood.",
    };
    render_html(
        &state,
        "blog_index.html",
        context! { page, blog_posts => published, tags, years, breadcrumbs },
        &request,
    )
}

async fn by_tag(
    State(state): State<AppState>,
    AxumPath(tag): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let published = posts::published(&state.posts);
    let filtered: Vec<Post> = published
        .iter()
        .filter(|p| p.tags.contains(&tag))
        .cloned()
        .collect();
    if filtered.is_empty() {
        return Err(AppError::not_found());
    }
    let extra_posts: Option<Vec<Post>> = if filtered.len() < 5 {
        Some(
            published
                .iter()
                .filter(|p| !p.tags.contains(&tag))
                .take(4)
                .cloned()
                .collect(),
        )
    } else {
        None
    };
    let tags = posts::collect_tags(&published);
    let years = posts::collect_years(&published);
    let active_tag = context! { name => &tag, slug => &tag };
    let page = context! {
        title => format!("Tag: {tag}"),
        slug => format!("tag-{tag}"),
        description => format!("Posts tagged {tag}"),
    };
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
        "blog_index.html",
        context! { page, blog_posts => filtered, extra_posts, active_tag, tags, years, breadcrumbs },
        &request,
    )
}

async fn by_year(
    State(state): State<AppState>,
    AxumPath(year): AxumPath<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let request = build_request(&uri, &headers);
    let published = posts::published(&state.posts);
    let filtered: Vec<Post> = published
        .iter()
        .filter(|p| p.date.starts_with(&year))
        .cloned()
        .collect();
    if filtered.is_empty() {
        return Err(AppError::not_found());
    }
    let extra_posts: Option<Vec<Post>> = if filtered.len() < 5 {
        Some(
            published
                .iter()
                .filter(|p| !p.date.starts_with(&year))
                .take(4)
                .cloned()
                .collect(),
        )
    } else {
        None
    };
    let tags = posts::collect_tags(&published);
    let years = posts::collect_years(&published);
    let page = context! {
        title => format!("Year: {year}"),
        slug => format!("year-{year}"),
        description => format!("Posts from {year}"),
    };
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
        "blog_index.html",
        context! { page, blog_posts => filtered, extra_posts, active_year => &year, tags, years, breadcrumbs },
        &request,
    )
}

async fn post_redirect(AxumPath(slug): AxumPath<String>) -> Redirect {
    Redirect::permanent(&format!("/posts/{slug}/"))
}

async fn post_pdf_redirect(AxumPath(slug): AxumPath<String>) -> Redirect {
    Redirect::permanent(&format!("/posts/{slug}/pdf/"))
}

async fn post_md_redirect(AxumPath(slug): AxumPath<String>) -> Redirect {
    Redirect::permanent(&format!("/posts/{slug}/md/"))
}
