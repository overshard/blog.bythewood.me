# blog.bythewood.me

Personal blog, served by a single Rust axum binary. Self-contained: posts,
templates, Vite-built static assets, and the binary all live here.

## Stack

| Concern         | Crate / Tool               |
|-----------------|----------------------------|
| Web framework   | axum + tokio               |
| Template engine | minijinja                  |
| Markdown        | comrak                     |
| PDF             | headless Chromium          |
| Static assets   | Vite + Bun                 |

Crate selection rationale (short version): axum is the most-pulled async
framework, minijinja is the only Rust engine that accepts upstream Jinja2
syntax, comrak's `partial-formatter` story is the closest match to
Mistune's renderer-override pattern.

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
│   └── pdf.rs        # chrome-headless-shell subprocess
├── templates/                    # jinja2 source (minijinja-compatible)
├── content/                      # markdown source
│   ├── posts/        # markdown posts with YAML frontmatter
│   └── images/       # served at /content/images/
├── frontend/                     # JS pipeline
│   ├── package.json, bun.lock, vite.config.js
│   ├── static_src/   # SCSS, JS, font imports
│   └── node_modules/ # gitignored
├── dist/                         # vite build output (gitignored, rebuilt by Docker, served at /static/)
└── target/                       # cargo build output (gitignored)
```

The binary reads `templates/`, `dist/`, and `content/` from the current
working directory. Override with `BLOG_ROOT=<path>`.

## Running

Dev (Vite watch + cargo run, port 8000):

```sh
make run
```

Production build (Vite assets + release binary):

```sh
make build
make start
```

Override port: `PORT=8001 make start`.

## Bench

`bench/run.sh` runs an `oha` load test sweep across the main routes. By
default it compares against a Flask server on port 8002 (the original
`blog.bythewood.me`).

## Caveats

- **PDF cold start.** Every PDF request spawns a fresh chromium process
  (~500 ms first hit). A persistent chromium with a remote-debugging port
  would close this gap but adds significant code.
- **Posts loaded once at startup.** Add a post, restart the process.
