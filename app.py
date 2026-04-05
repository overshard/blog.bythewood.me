"""
app.py

A simple Flask blog powered by markdown files.
"""

import html
import math
import os
import random
from datetime import date, datetime

import mistune
from flask import (
    Flask,
    Response,
    abort,
    jsonify,
    render_template,
    request,
    send_from_directory,
    url_for,
)
from weasyprint import HTML

app = Flask(__name__)

CONTENT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "content")


# -- Markdown rendering -------------------------------------------------------


class BlogRenderer(mistune.HTMLRenderer):
    def block_code(self, code, info=None, **attrs):
        lang = info or ""
        if lang == "html":
            lang = "htmlmixed"
        escaped = html.escape(code)
        return (
            f'<div class="block-code">'
            f'<textarea data-language="{lang}">{escaped}</textarea>'
            f"</div>\n"
        )

    def image(self, text, url, title=None):
        if url.startswith("images/"):
            url = url[len("images/"):]
        return (
            f'<div class="block-image">'
            f'<img src="/content/images/{url}" class="rounded" alt="{text}">'
            f"</div>\n"
        )

    def paragraph(self, text):
        if text.strip().startswith('<div class="block-image">'):
            return text
        return f'<div class="block-rich-text"><p>{text}</p></div>\n'

    def list(self, text, ordered, **attrs):
        tag = "ol" if ordered else "ul"
        return f'<div class="block-rich-text"><{tag}>{text}</{tag}></div>\n'

    def heading(self, text, level, **attrs):
        return f'<div class="block-rich-text"><h{level}>{text}</h{level}></div>\n'

    def block_quote(self, text):
        return f'<div class="block-rich-text"><blockquote>{text}</blockquote></div>\n'

    def thematic_break(self):
        return '<div class="block-rich-text"><hr></div>\n'


markdown = mistune.create_markdown(renderer=BlogRenderer(), plugins=["strikethrough"])


class PDFRenderer(BlogRenderer):
    def block_code(self, code, info=None, **attrs):
        lang = info or ""
        escaped = html.escape(code)
        return (
            f'<div class="block-code">'
            f'<pre><code class="language-{lang}">{escaped}</code></pre>'
            f"</div>\n"
        )


markdown_pdf = mistune.create_markdown(renderer=PDFRenderer(), plugins=["strikethrough"])


# -- Content loading -----------------------------------------------------------


def parse_frontmatter(text):
    """Parse YAML-like frontmatter from a markdown file."""
    if not text.startswith("---"):
        return {}, text
    end = text.find("---", 3)
    if end == -1:
        return {}, text
    meta = {}
    for line in text[3:end].strip().split("\n"):
        if ": " in line:
            key, value = line.split(": ", 1)
            meta[key.strip()] = value.strip()
    body = text[end + 3 :].strip()
    return meta, body


def load_posts():
    """Load all markdown posts from content/posts/."""
    posts = []
    posts_dir = os.path.join(CONTENT_DIR, "posts")
    if not os.path.exists(posts_dir):
        return posts

    for filename in os.listdir(posts_dir):
        if not filename.endswith(".md"):
            continue
        filepath = os.path.join(posts_dir, filename)
        with open(filepath) as f:
            text = f.read()
        meta, body = parse_frontmatter(text)

        tags = [t.strip() for t in meta.get("tags", "").split(",") if t.strip()]
        post_date = meta.get("date", "")
        publish_date = meta.get("publish_date", post_date)

        post = {
            "filename": filename,
            "title": meta.get("title", ""),
            "slug": meta.get("slug", filename[:-3]),
            "date": post_date,
            "publish_date": publish_date,
            "tags": tags,
            "description": meta.get("description", ""),
            "cover_image": meta.get("cover_image", ""),
            "body_html": markdown(body),
            "body_html_pdf": markdown_pdf(body),
            "read_time": max(1, math.ceil(len(body.split()) / 200)),
        }
        posts.append(post)

    posts.sort(key=lambda p: p["date"], reverse=True)
    return posts


POSTS = load_posts()
POSTS_BY_SLUG = {p["slug"]: p for p in POSTS}


@app.before_request
def reload_posts_in_debug():
    global POSTS, POSTS_BY_SLUG
    if app.debug:
        POSTS = load_posts()
        POSTS_BY_SLUG = {p["slug"]: p for p in POSTS}


# -- Helpers -------------------------------------------------------------------


def get_published_posts():
    """Return posts where publish_date <= today."""
    today = date.today().isoformat()
    return [p for p in POSTS if p["publish_date"] <= today]


def get_tags(posts):
    """Return sorted list of unique tags with counts and URLs."""
    counts = {}
    for post in posts:
        for tag in post["tags"]:
            counts[tag] = counts.get(tag, 0) + 1
    return sorted(
        [{"name": t, "slug": t, "count": c, "url": url_for("blog_tag", tag=t)} for t, c in counts.items()],
        key=lambda t: t["name"],
    )


def get_years(posts):
    """Return sorted list of unique years."""
    return sorted(set(p["date"][:4] for p in posts if p["date"]), reverse=True)


def get_related(post, posts, count=3):
    """Find posts with the most overlapping tags."""
    if not post["tags"]:
        return posts[:count]
    scored = []
    for p in posts:
        if p["slug"] == post["slug"]:
            continue
        overlap = len(set(post["tags"]) & set(p["tags"]))
        if overlap > 0:
            scored.append((overlap, p))
    scored.sort(key=lambda x: x[0], reverse=True)
    related = [p for _, p in scored[:count]]
    if len(related) < count:
        related_slugs = {p["slug"] for p in related}
        for p in posts:
            if p["slug"] != post["slug"] and p["slug"] not in related_slugs:
                related.append(p)
                if len(related) >= count:
                    break
    return related


# -- Context processor ---------------------------------------------------------


@app.errorhandler(404)
def page_not_found(e):
    return render_template("404.html", page={"title": "404", "description": "Page not found"}), 404


@app.context_processor
def inject_globals():
    posts = get_published_posts()
    return {
        "nav_items": get_tags(posts),
        "now": datetime.now(),
        "debug": app.debug,
    }


# -- Routes --------------------------------------------------------------------


@app.route("/")
def index():
    posts = get_published_posts()
    latest_post = posts[0] if posts else None
    rest = [p for p in posts if p != latest_post]
    random_posts = random.sample(rest, min(3, len(rest)))
    return render_template(
        "home.html",
        page={"title": "Isaac Bythewood's Blog", "slug": "home", "description": "Writing about webdev, infrastructure, security, and tooling by Isaac Bythewood, a Senior Solutions Architect in Elkin, NC."},
        latest_post=latest_post,
        random_blog_posts=random_posts,
    )


@app.route("/blog/")
def blog_index():
    posts = get_published_posts()
    return render_template(
        "blog_index.html",
        page={"title": "Blog", "slug": "blog", "description": "Posts on webdev, coding, security, and sysadmin by Isaac Bythewood."},
        blog_posts=posts,
        tags=get_tags(posts),
        years=get_years(posts),
        breadcrumbs=[{"title": "Home", "url": url_for("index")}],
    )


@app.route("/blog/<slug>/")
def blog_post(slug):
    post = POSTS_BY_SLUG.get(slug)
    if not post or post["publish_date"] > date.today().isoformat():
        abort(404)
    posts = get_published_posts()
    return render_template(
        "blog_post.html",
        page=post,
        post=post,
        related_posts=get_related(post, posts),
        breadcrumbs=[
            {"title": "Home", "url": url_for("index")},
            {"title": "Blog", "url": url_for("blog_index")},
        ],
    )


@app.route("/blog/<slug>/pdf/")
def blog_post_pdf(slug):
    post = POSTS_BY_SLUG.get(slug)
    if not post or post["publish_date"] > date.today().isoformat():
        abort(404)
    html_content = render_template("blog_post_pdf.html", post=post)
    pdf = HTML(
        string=html_content,
        base_url=request.url_root,
    ).write_pdf()
    return Response(
        pdf,
        mimetype="application/pdf",
        headers={"Content-Disposition": f'filename="{post["slug"]}.pdf"'},
    )


@app.route("/blog/<slug>/md/")
def blog_post_md(slug):
    post = POSTS_BY_SLUG.get(slug)
    if not post or post["publish_date"] > date.today().isoformat():
        abort(404)
    filepath = os.path.join(CONTENT_DIR, "posts", post["filename"])
    with open(filepath) as f:
        content = f.read()
    return Response(
        content,
        mimetype="text/markdown",
        headers={"Content-Disposition": f'filename="{post["slug"]}.md"'},
    )


@app.route("/blog/tag/<tag>/")
def blog_tag(tag):
    posts = get_published_posts()
    filtered = [p for p in posts if tag in p["tags"]]
    if not filtered:
        abort(404)
    extra_posts = None
    if len(filtered) < 5:
        extra_posts = [p for p in posts if tag not in p["tags"]][:4]
    return render_template(
        "blog_index.html",
        page={"title": f"Tag: {tag}", "slug": f"tag-{tag}", "description": f"Posts tagged {tag}"},
        blog_posts=filtered,
        extra_posts=extra_posts,
        active_tag={"name": tag, "slug": tag},
        tags=get_tags(posts),
        years=get_years(posts),
        breadcrumbs=[
            {"title": "Home", "url": url_for("index")},
            {"title": "Blog", "url": url_for("blog_index")},
        ],
    )


@app.route("/blog/year/<int:year>/")
def blog_year(year):
    posts = get_published_posts()
    filtered = [p for p in posts if p["date"].startswith(str(year))]
    if not filtered:
        abort(404)
    extra_posts = None
    if len(filtered) < 5:
        extra_posts = [p for p in posts if not p["date"].startswith(str(year))][:4]
    return render_template(
        "blog_index.html",
        page={"title": f"Year: {year}", "slug": f"year-{year}", "description": f"Posts from {year}"},
        blog_posts=filtered,
        extra_posts=extra_posts,
        active_year=str(year),
        tags=get_tags(posts),
        years=get_years(posts),
        breadcrumbs=[
            {"title": "Home", "url": url_for("index")},
            {"title": "Blog", "url": url_for("blog_index")},
        ],
    )


@app.route("/search/")
def search():
    q = request.args.get("q", "")
    posts = get_published_posts()
    results = []
    random_posts = None
    if q:
        ql = q.lower()
        for p in posts:
            if (
                ql in p["title"].lower()
                or ql in p["description"].lower()
                or any(ql in t.lower() for t in p["tags"])
            ):
                results.append(p)
    else:
        random_posts = random.sample(posts, min(6, len(posts)))
    return render_template(
        "search.html",
        page={"title": "Search", "slug": "search", "description": "Search posts on webdev, coding, security, and sysadmin."},
        results=results,
        random_posts=random_posts,
        q=q,
        breadcrumbs=[{"title": "Home", "url": url_for("index")}],
    )


@app.route("/search/live/")
def search_live():
    q = request.args.get("q", "")
    posts = get_published_posts()
    results = []
    if q:
        ql = q.lower()
        for p in posts:
            if (
                ql in p["title"].lower()
                or ql in p["description"].lower()
                or any(ql in t.lower() for t in p["tags"])
            ):
                results.append(
                    {
                        "title": p["title"],
                        "description": p["description"],
                        "url": url_for("blog_post", slug=p["slug"]),
                    }
                )
                if len(results) >= 5:
                    break
    return jsonify(results)


@app.route("/content/images/<path:filename>")
def content_images(filename):
    return send_from_directory(os.path.join(CONTENT_DIR, "images"), filename)


@app.route("/og/<slug>.svg")
def og_image(slug):
    post = POSTS_BY_SLUG.get(slug)
    if post:
        title = post["title"]
        tags = post["tags"]
    else:
        title = "Isaac Bythewood's Blog"
        tags = []
    # Wrap title into lines (~35 chars each)
    words = title.split()
    lines = []
    current = ""
    for word in words:
        if current and len(current) + len(word) + 1 > 35:
            lines.append(current)
            current = word
        else:
            current = f"{current} {word}" if current else word
    if current:
        lines.append(current)
    return (
        render_template(
            "og.svg",
            title_lines=lines[:3],
            tags=tags,
        ),
        200,
        {"Content-Type": "image/svg+xml"},
    )


@app.route("/favicon.ico")
def favicon():
    return render_template("favicon.svg"), 200, {"Content-Type": "image/svg+xml"}


@app.route("/robots.txt")
def robots():
    return render_template("robots.txt"), 200, {"Content-Type": "text/plain"}


@app.route("/sitemap.xml")
def sitemap():
    posts = get_published_posts()
    tags = get_tags(posts)
    years = get_years(posts)
    # Compute lastmod for tag and year pages
    tag_lastmod = {}
    year_lastmod = {}
    for p in posts:
        for t in p["tags"]:
            if t not in tag_lastmod or p["date"] > tag_lastmod[t]:
                tag_lastmod[t] = p["date"]
        y = p["date"][:4]
        if y not in year_lastmod or p["date"] > year_lastmod[y]:
            year_lastmod[y] = p["date"]
    return (
        render_template(
            "sitemap.xml",
            posts=posts,
            tags=tags,
            years=years,
            tag_lastmod=tag_lastmod,
            year_lastmod=year_lastmod,
        ),
        200,
        {"Content-Type": "application/xml"},
    )
