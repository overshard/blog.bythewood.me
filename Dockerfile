FROM python:3.13-alpine

COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

RUN apk add --update --no-cache \
      nodejs yarn \
      pango fontconfig font-noto font-jetbrains-mono

WORKDIR /app

COPY pyproject.toml uv.lock package.json yarn.lock ./
RUN yarn install && uv sync --frozen --no-dev

COPY . .

ENV PATH="/app/.venv/bin:/app/node_modules/.bin:$PATH"

RUN webpack --config webpack.config.js --mode production

RUN addgroup -S -g 1000 app && \
    adduser -S -h /app -s /sbin/nologin -u 1000 -G app app && \
    chown -R app:app /app
USER app
