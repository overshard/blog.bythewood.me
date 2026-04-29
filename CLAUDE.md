# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A personal blog (blog.bythewood.me) built as a Flask app that renders markdown files. No database — blog posts are `.md` files in `content/posts/` with YAML frontmatter. Uses WeasyPrint for PDF export and Mistune for markdown rendering.

## Commands

- **Dev server:** `make run` (runs Vite watch + Flask dev server concurrently)
- **Build frontend:** `make build` (or `bun run build`)
- **Docker build:** `sudo docker build .`

There are no tests or linters configured.

## Architecture

**Backend:** Single-file Flask app (`app.py`). Posts are loaded from `content/posts/*.md` at startup (reloaded per-request in debug mode). Each post has frontmatter fields: title, slug, date, publish_date, tags, description, cover_image. Posts with `publish_date` in the future are hidden.

**Frontend pipeline:** Vite builds `static_src/` → `static/`. Entry point is `static_src/index.js` which imports SCSS and JS. Output filenames are content-hashed (`base-[hash].js`, `base-[hash].css`) and a Vite manifest (`static/.vite/manifest.json`) is read at runtime so templates can resolve the hashed names for cache busting. Uses Bootstrap 5, CodeMirror (syntax highlighting in posts), and Monaspace Argon font.

**Templates:** Jinja2 templates in `templates/`. `base.html` is the layout. Blog post content is rendered through a custom `BlogRenderer` (Mistune) that wraps blocks in `div.block-*` classes. A separate `PDFRenderer` exists for the PDF export route.

**Content:** `content/posts/` for markdown posts, `content/images/` for images served at `/content/images/`.

## Tooling

- **Python deps:** managed with `uv` (see `pyproject.toml`, `uv.lock`)
- **JS deps:** managed with `bun` (see `package.json`, `bun.lock`)
- **Production:** Docker (Alpine-based) + Gunicorn, deployed via `docker-compose`

## Key Routes

- `/posts/<slug>/` — single post (old `/blog/<slug>/` 301-redirects here)
- `/posts/<slug>/pdf/` — PDF export via WeasyPrint
- `/posts/<slug>/md/` — raw markdown download
- `/blog/` — post index (also `/blog/tag/<tag>/` and `/blog/year/<year>/`)
- `/search/live/` — JSON endpoint for live search
- `/og/<slug>.svg` — dynamic OG image generation
