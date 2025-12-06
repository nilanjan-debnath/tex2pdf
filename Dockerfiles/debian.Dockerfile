# syntax=docker/dockerfile:1

# ---------------------------------------------------
# 1. PLANNER STAGE
# ---------------------------------------------------
FROM rust:slim-bookworm AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


# ---------------------------------------------------
# 2. BUILDER STAGE
# ---------------------------------------------------
FROM rust:slim-bookworm AS build

# Install all required development libraries for Tectonic
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libfontconfig1-dev \
    libgraphite2-dev \
    libharfbuzz-dev \
    libicu-dev \
    zlib1g-dev \
    g++ \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef sccache --locked

ENV RUSTC_WRAPPER=sccache \
    SCCACHE_DIR=/sccache

WORKDIR /app

# Copy ONLY the recipe from the planner stage
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies with external-harfbuzz feature
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json --features tectonic/external-harfbuzz

# NOW copy the actual source code
COPY . .

# Build the application with external-harfbuzz feature
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release --bin tex2pdf --features tectonic/external-harfbuzz


# ---------------------------------------------------
# 3. RUNTIME STAGE
# ---------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies for Tectonic
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    libfontconfig1 \
    libgraphite2-3 \
    libharfbuzz0b \
    libicu72 \
    && rm -rf /var/lib/apt/lists/*

# Creating a non root user (Debian syntax)
RUN groupadd -r appgroup && useradd -r -g appgroup appuser

ENV XDG_CACHE_HOME=/app/.cache
WORKDIR /app
RUN mkdir -p $XDG_CACHE_HOME && chown -R appuser:appgroup /app

# Copy the binary files from builder stage
COPY --from=build --chown=appuser:appgroup /app/target/release/tex2pdf /usr/local/bin/app

# Switch to non root user
USER appuser

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=180s --start-period=300s --retries=3 \
  CMD curl -f http://localhost:3000/healthz || exit 1

ENTRYPOINT ["/usr/local/bin/app"]