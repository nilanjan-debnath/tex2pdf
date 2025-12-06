# syntax=docker/dockerfile:1

# ---------------------------------------------------
# 1. PLANNER STAGE
# ---------------------------------------------------
FROM rust:1.91.1-alpine3.22 AS planner
WORKDIR /app
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


# ---------------------------------------------------
# 2. BUILDER STAGE
# ---------------------------------------------------
FROM rust:1.91.1-alpine3.22 AS build

# Install build dependencies (no tectonic deps needed since we use CLI)
RUN apk add --no-cache \
    build-base \
    openssl-dev \
    openssl-libs-static \
    pkgconfig

# Ensure OpenSSL is linked statically
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include

RUN cargo install cargo-chef sccache --locked

ENV RUSTC_WRAPPER=sccache \
    SCCACHE_DIR=/sccache

WORKDIR /app

# Copy ONLY the recipe from the planner stage
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# NOW copy the actual source code
COPY . .

# Build the application
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release --bin tex2pdf


# ---------------------------------------------------
# 3. RUNTIME STAGE
# ---------------------------------------------------
FROM alpine:3.22 AS runtime

# Install runtime dependencies including tectonic CLI
RUN apk add --no-cache \
    tectonic \
    fontconfig \
    libgcc \
    libstdc++

# Creating a non root user (Alpine syntax)
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

WORKDIR /app
ENV XDG_CACHE_HOME=/app/.cache
RUN mkdir -p $XDG_CACHE_HOME && chown -R appuser:appgroup /app

# Copy the binary files from builder stage
COPY --from=build --chown=appuser:appgroup /app/target/release/tex2pdf /usr/local/bin/app

# Switch to non root user
USER appuser

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/app"]