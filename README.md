# blog.bythewood.me

Personal blog, served by a single Rust axum binary. Self-contained: posts, templates, Vite-built static assets, and the binary all live here. Markdown content in `content/posts/`, no database.

## Features

- Markdown posts with YAML frontmatter (title, slug, date, publish_date, tags, description, cover_image)
- Server-rendered tag and year archives
- Server-rendered + live JSON search (`/search/` and `/search/live/`)
- Per-post PDF export via embedded Typst (no chromium subprocess)
- Per-post raw markdown download
- Dynamic OG image generation per post
- Single-binary deploy via `git push server master`

## Stack

| Concern         | Crate / Tool                 |
|-----------------|------------------------------|
| Web framework   | axum + tokio                 |
| Template engine | minijinja                    |
| Markdown        | comrak                       |
| PDF             | embedded Typst (no chromium) |
| Static assets   | Vite + Bun                   |

Crate selection rationale: axum is the most-pulled async framework, minijinja is the only Rust engine that accepts upstream Jinja2 syntax, comrak's `partial-formatter` story is the closest match to Mistune's renderer-override pattern.

## System dependencies

Local dev needs all of these on your `PATH`:

| Tool | Why | Version |
|---|---|---|
| `rustc` / `cargo` | Build the axum binary | 2021 edition, current stable is fine (1.70+) |
| `bun` | Frontend deps + Vite | 1.x |
| `make` | Run the dev/build targets | any |

The Docker build (see `Dockerfile`) reproduces this on `rust:alpine` + `alpine:3.23`. If you only care about Docker, you do not need any of the above on the host. Runtime image installs `font-jetbrains-mono`, `ttf-dejavu`, `ttf-liberation`, and `fontconfig` so the embedded Typst renderer can find body sans, mono, and fallback fonts.

## Quickstart

```sh
cp samplefiles/env.sample .env
make
```

`make` (alias `make run`) installs frontend deps if needed, then runs Vite watch and `cargo run` concurrently on port 8000. Visit http://localhost:8000.

## Configuration

All config comes from `.env` (loaded via `dotenvy`):

| Variable | Required | Purpose |
|---|---|---|
| `PORT` | no (default `8000`) | HTTP listen port |
| `BLOG_ROOT` | no | Override the project root (where `templates/`, `dist/`, and `content/` are read from) |

## Make targets

| Target | What it does |
|---|---|
| `make run` (default) | Vite watch + `cargo run` on port 8000 |
| `make build` | Vite assets + release binary (`target/release/blog`) |
| `make start` | Run the release binary (after `make build`) |
| `make bench` | `oha` load test sweep across the main routes. Compares against a Flask server on port 8002 (the original `blog.bythewood.me`) if running |
| `make push` | `git push` to every configured remote |
| `make clean` | Remove `target/`, `dist/`, and `frontend/node_modules/` |

There are no tests or linters configured.

## Layout

```
blog.bythewood.me/
├── Cargo.toml, Cargo.lock        # rust deps
├── Makefile, README.md, bench/   # top-level
├── src/                          # rust source
│   ├── main.rs       # tiny entry: server boot
│   ├── app.rs        # AppState + Router assembly
│   ├── render.rs     # render_html helper
│   ├── middleware.rs # request log
│   ├── routes/       # blog, post, search, seo, errors
│   ├── posts.rs      # frontmatter + post loading
│   ├── markdown.rs   # comrak custom renderer
│   ├── templates.rs  # minijinja env, url_for, vite_asset, Jinja2-compat formatter
│   └── pdf.rs        # embedded Typst renderer
├── templates/                    # jinja2 source (minijinja-compatible) + blog_post.typ
├── content/                      # markdown source
│   ├── posts/        # markdown posts with YAML frontmatter
│   └── images/       # served at /content/images/
├── frontend/                     # JS pipeline (package.json, vite.config.js, static_src/)
├── dist/                         # vite build output (gitignored, served at /static/)
├── target/                       # cargo build output (gitignored)
└── samplefiles/                  # Caddyfile.sample, env.sample, post-receive.sample
```

The binary reads `templates/`, `dist/`, and `content/` from the current working directory. Override with `BLOG_ROOT=<path>`.

## Deploy

Production runs on Docker. The standard flow is `git push server master` to a remote whose post-receive hook runs `docker compose up --build --detach`. Sample files in `samplefiles/`:

- `Caddyfile.sample`: reverse proxy with TLS
- `env.sample`: the `.env` shown above
- `post-receive.sample`: the git hook

See [CLAUDE.md](CLAUDE.md) for the full architecture rundown, route table, and PDF pipeline details.

## Caveats

- **Posts loaded once at startup.** Add a post, restart the process.
