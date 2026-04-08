FROM python:3.13-alpine

COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/
COPY --from=oven/bun:alpine /usr/local/bin/bun /usr/local/bin/bun

RUN apk add --update --no-cache \
      pango fontconfig font-noto font-jetbrains-mono

WORKDIR /app

COPY pyproject.toml uv.lock package.json bun.lock ./
RUN bun install --frozen-lockfile && uv sync --frozen --no-dev

COPY . .

ENV PATH="/app/.venv/bin:/app/node_modules/.bin:$PATH"

RUN bun run build

RUN addgroup -S -g 1000 app && \
    adduser -S -h /app -s /sbin/nologin -u 1000 -G app app && \
    chown -R app:app /app
USER app
