# Stage 1: Build SolidJS frontend
FROM node:22-alpine AS frontend
WORKDIR /app/frontend
COPY ferrishost-web/frontend/package.json ferrishost-web/frontend/package-lock.json ./
RUN npm ci
COPY ferrishost-web/frontend/ ./
RUN npm run build

# Stage 2: Build Rust backend
FROM rust:1.87-slim AS backend
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY ferrishost-core/ ferrishost-core/
COPY ferrishost-web/ ferrishost-web/
RUN cargo build --release -p ferrishost-web

# Stage 3: Final image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=backend /app/target/release/ferrishost-web /usr/local/bin/ferrishost-web
COPY --from=frontend /app/frontend/dist /srv/static

ENV STATIC_DIR=/srv/static
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/ferrishost-web"]
