# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A personal blog (blog.bythewood.me) built as a Rust axum app that renders markdown files. No database: blog posts are `.md` files in `content/posts/` with YAML frontmatter. Uses Typst (embedded as a library) for PDF export and comrak for markdown rendering.

This is a Rust port of the original Flask version (now deleted). Performance: ~30-50x lower memory, ~10-20x higher RPS, sub-millisecond per-request latency in release mode.

## Commands

- **Dev server:** `make run` (runs Vite watch + cargo run concurrently on port 8000)
- **Production build:** `make build` (Vite assets + release binary)
- **Run release binary:** `make start`
- **Bench sweep:** `make bench` (oha load test, used to compare against the old Flask version when both run side-by-side)
- **Docker build:** `sudo docker build .`

There are no tests or linters configured.

## Architecture

**Backend:** Single-binary axum app (`src/main.rs`). Posts are loaded from `content/posts/*.md` once at startup. Each post has frontmatter fields: title, slug, date, publish_date, tags, description, cover_image. Posts with `publish_date` in the future are hidden.

**Frontend pipeline:** Vite (run from `frontend/`) builds `frontend/static_src/` → `dist/`. Entry point is `frontend/static_src/index.js` which imports SCSS and JS. Output filenames are content-hashed (`base-[hash].js`, `base-[hash].css`) and a Vite manifest (`dist/.vite/manifest.json`) is read at runtime so templates can resolve the hashed names. Uses Bootstrap 5, CodeMirror, and Monaspace Argon font.

**Templates:** Jinja2 templates in `templates/` rendered by minijinja (Jinja2-faithful Rust engine by Armin Ronacher). `base.html` is the layout. Markdown post content is rendered through comrak with a custom renderer (`src/markdown.rs`) that wraps blocks in `div.block-*` classes, mirroring the original Mistune renderer pattern.

**PDF generation:** `src/pdf.rs` embeds the Typst compiler (`typst` + `typst-pdf`) as a library, no chromium subprocess. At startup `PdfRenderer::new` runs `typst-kit`'s font searcher to discover system fonts (and bundle a few embedded ones via the `embed-fonts` feature). Per request: comrak walks the post's markdown AST to a Typst markup string (`markdown::render_typst`), `main.rs::build_typst_source` wraps it in a `#import "/templates/blog_post.typ": render` + `#render(title: ..., body: [...])` invocation, and `PdfRenderer::render` compiles that source to a `PagedDocument` then calls `typst_pdf::pdf` for the bytes. The `World` impl resolves absolute paths against the project root, so `image("/content/images/foo.webp")` reads `content/images/foo.webp` from disk.

**Manifest reload:** `templates.rs::build_env` re-reads `dist/.vite/manifest.json` per `vite_asset()` call in debug builds (so Vite watcher rebuilds are picked up immediately). Release builds load it once at startup. Gated on `cfg(debug_assertions)`.

**Request logging:** custom middleware in `src/main.rs::log_requests` prints `time METHOD STATUS latency path` per request, with ANSI-colored status codes (green 2xx, cyan 3xx, yellow 4xx, red 5xx). Always-on, costs sub-microsecond per request. The `.layer()` is applied after all routes so it covers `nest_service` static-file mounts and the `fallback` 404 handler.

**Content:** `content/posts/` for markdown posts, `content/images/` for images served at `/content/images/`.

## Layout

```
blog.bythewood.me/
├── Cargo.toml, Cargo.lock        # rust deps
├── Makefile, README.md, bench/   # top-level
├── src/                          # rust source
│   ├── main.rs       # axum routes
│   ├── posts.rs      # frontmatter + post loading
│   ├── markdown.rs   # comrak custom renderer
│   ├── templates.rs  # minijinja env, url_for, vite_asset, Jinja2-compat formatter
│   └── pdf.rs        # typst World + render entry point
├── templates/                    # jinja2 source (minijinja-compatible)
├── content/                      # markdown source (posts + images)
├── frontend/                     # JS pipeline (package.json, vite.config.js, static_src/, node_modules/)
├── dist/                         # vite build output (gitignored, served at /static/)
├── target/                       # cargo build output (gitignored)
└── samplefiles/                  # Caddyfile.sample, env.sample, post-receive.sample
```

The binary reads `templates/`, `dist/`, and `content/` from the current working directory by default. Override the project root with `BLOG_ROOT=<path>`.

## Tooling

- **Rust deps:** managed with `cargo` (see `Cargo.toml`, `Cargo.lock`)
- **JS deps:** managed with `bun`, run from `frontend/` (see `frontend/package.json`, `frontend/bun.lock`)
- **Production:** Docker (Alpine-based multi-stage, `rust:alpine` builder + `alpine:3.23` runtime), deployed via `docker-compose`. Runtime image installs `font-jetbrains-mono`, `ttf-dejavu`, `ttf-liberation`, and `fontconfig` so the embedded Typst renderer can find a body sans, mono, and fallback fonts.

## Key Routes

- `/posts/<slug>/`: single post (old `/blog/<slug>/` 301-redirects here)
- `/posts/<slug>/pdf/`: PDF export via embedded Typst (template at `templates/blog_post.typ`)
- `/posts/<slug>/md/`: raw markdown download
- `/blog/`: post index (also `/blog/tag/<tag>/` and `/blog/year/<year>/`)
- `/search/?q=...`: server-rendered search page
- `/search/live/?q=...`: JSON endpoint for live search
- `/og/<slug>.svg`: dynamic OG image generation
