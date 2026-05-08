# syntax=docker/dockerfile:1
# ----- builder -----
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

COPY --from=oven/bun:alpine /usr/local/bin/bun /usr/local/bin/bun

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY frontend ./frontend

RUN cd frontend && bun install --frozen-lockfile && bun run build
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/blog /app/blog

# ----- runtime -----
FROM alpine:3.23

RUN apk add --no-cache \
    font-jetbrains-mono ttf-dejavu ttf-liberation fontconfig

WORKDIR /app

COPY --from=builder /app/blog ./blog
COPY --from=builder /app/dist ./dist
COPY templates ./templates
COPY content ./content

RUN addgroup -S -g 1000 app && \
    adduser -S -h /app -s /sbin/nologin -u 1000 -G app app && \
    chown -R app:app /app
USER app

ENV PORT=8000
EXPOSE 8000

CMD ["./blog"]
