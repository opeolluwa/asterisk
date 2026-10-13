FROM rust:1.97.0-slim-bookworm

ENV CARGO_TARGET_DIR=/cargo-target

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        libgcc-s1 \
        libssl-dev \
        pkg-config \
        clang \
        libclang-dev \
        zlib1g-dev \
        llvm-dev \
        build-essential; \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash appuser

RUN cargo install cargo-watch

WORKDIR /app

COPY macros/ ./macros/
COPY . .

WORKDIR /app

RUN cargo build

RUN chown -R appuser:appuser /app /cargo-target /usr/local/cargo/registry

USER appuser

VOLUME ["/cargo-target", "/usr/local/cargo/registry"]

CMD ["cargo", "watch", \
     "--why", \
     "--clear", \
     "--quiet", \
     "--ignore", ".git/", \
     "--ignore", "target/", \
     "-w", "src", \
     "-w", "migration", \
     "-w", "templates", \
     "-w", "/macros", \
     "-x", "check", \
     "-x", "run"]