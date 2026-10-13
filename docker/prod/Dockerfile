# Stage 1: Rust build
FROM rust:1.97.0-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY macros/Cargo.toml macros/Cargo.toml
COPY migration/Cargo.toml migration/Cargo.toml

COPY macros/ macros/
COPY migration/ migration/
COPY src/ src/
COPY templates/ templates/

RUN cargo build --release --bin asterisk --bin asterisk-cli \
    && strip target/release/asterisk \
    && strip target/release/asterisk-cli


# Stage 2: Runtime
FROM debian:bookworm-slim

RUN groupadd --system app \
    && useradd --system \
        --gid app \
        --home-dir /app \
        --shell /usr/sbin/nologin \
        app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder --chown=app:app \
    /app/target/release/asterisk ./asterisk

COPY --from=builder --chown=app:app \
    /app/target/release/asterisk-cli ./asterisk-cli

COPY --from=builder --chown=app:app \
    /app/templates ./templates

USER app:app

EXPOSE 8000

HEALTHCHECK \
    --interval=30s \
    --timeout=3s \
    --start-period=5s \
    --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

ENTRYPOINT ["./asterisk"]