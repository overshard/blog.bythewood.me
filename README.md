# blog.bythewood.me

My personal blog. It is a single Rust axum binary that renders Markdown files: posts, templates, Vite-built static assets, and the binary all live in this one repo. There is no database. To publish, you write a Markdown file and restart the process.

It started life as a Flask app. The Rust rewrite uses far less memory, serves far more requests, and answers in well under a millisecond per request in release mode, which is why it exists in this form.

## Features

- Markdown posts with YAML frontmatter (title, slug, date, publish_date, tags, description, cover_image)
- Server-rendered tag and year archives
- Server-rendered search plus a live JSON search endpoint (`/search/` and `/search/live/`)
- Per-post PDF export via embedded Typst (no chromium subprocess)
- Per-post raw markdown download
- Dynamic OG image generation per post
- Single-binary deploy via `git push server master`

## Tech stack

| Concern         | Crate / Tool                 |
|-----------------|------------------------------|
| Web framework   | axum + tokio                 |
| Template engine | minijinja                    |
| Markdown        | comrak                       |
| PDF             | embedded Typst (no chromium) |
| Static assets   | Vite + Bun                   |

Why these: axum is the most-used async framework, minijinja is the Rust engine that accepts upstream Jinja2 syntax unchanged, comrak's partial-formatter hook is the closest match to the Mistune renderer-override pattern the original Flask version used, and embedded Typst renders PDFs in-process without spawning a browser.

## Quickstart

```sh
cp samplefiles/env.sample .env
make
```

`make` (alias `make run`) installs frontend deps if needed, then runs Vite watch and `cargo run` concurrently on port 8000. Visit http://localhost:8000.

You need these on your `PATH` for local dev:

| Tool | Why | Version |
|---|---|---|
| `rustc` / `cargo` | Build the axum binary | 2021 edition, stable 1.70+ |
| `bun` | Frontend deps + Vite | 1.x |
| `make` | Run the dev/build targets | any |

If you only deploy via Docker you do not need any of these on the host; the `Dockerfile` reproduces the toolchain on `rust:alpine` + `alpine:3.23`.

## Writing a post

1. Create a Markdown file under `content/posts/`, for example `content/posts/my-post.md`.
2. Give it YAML frontmatter:

   ```yaml
   ---
   title: My Post
   slug: my-post
   date: 2026-06-19
   publish_date: 2026-06-19
   tags: [rust, notes]
   description: A short summary used in listings and OG images.
   cover_image: my-cover.webp
   ---
   ```

   Body Markdown follows the frontmatter.
3. Put any images in `content/images/`; they are served at `/content/images/`.
4. **Restart the process.** Posts are loaded once at startup, so a new or edited post only appears after a restart. A `publish_date` in the future hides the post until that date (and still needs a restart once the date has passed).

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
| `make bench` | `oha` load test sweep across the main routes. Compares against a Flask server on port 8002 if running |
| `make push` | `git push` to every configured remote |
| `make clean` | Remove `target/`, `dist/`, and `frontend/node_modules/` |

There are no tests or linters configured.

## Routes

- `/posts/<slug>/`: single post (old `/blog/<slug>/` 301-redirects here)
- `/posts/<slug>/pdf/`: PDF export
- `/posts/<slug>/md/`: raw markdown download
- `/blog/`: post index, plus `/blog/tag/<tag>/` and `/blog/year/<year>/`
- `/search/?q=...` and `/search/live/?q=...`: search
- `/og/<slug>.svg`: per-post OG image

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
│   ├── routes/       # home, blog, post, search, seo, errors
│   ├── posts.rs      # frontmatter + post loading
│   ├── markdown.rs   # comrak custom renderer
│   ├── templates.rs  # minijinja env, url_for, vite_asset, Jinja2-compat formatter
│   └── pdf.rs        # embedded Typst renderer
├── templates/                    # jinja2 source + blog_post.typ
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

Production runs on Docker. The flow is `git push server master` to a remote whose post-receive hook runs `docker compose up --build --detach`. The `alpine:3.23` runtime image installs `font-jetbrains-mono`, `ttf-dejavu`, `ttf-liberation`, and `fontconfig` so the embedded Typst renderer can find body, mono, and fallback fonts. Sample files in `samplefiles/`:

- `Caddyfile.sample`: reverse proxy with TLS
- `env.sample`: the `.env` shown above
- `post-receive.sample`: the git hook
