# syntax=docker/dockerfile:1

# ---------------------------------------------------
# 1. PLANNER STAGE
# ---------------------------------------------------
FROM rust:1.91.1-alpine3.22 AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


# ---------------------------------------------------
# 2. BUILDER STAGE
# ---------------------------------------------------
FROM rust:1.91.1-alpine3.22 AS build

# Install build dependencies
# UPDATED: Added '-static' packages. 
# Rust on Alpine (musl) prefers static linking. If pkg-config can't find 
# static libs (libharfbuzz.a), Tectonic tries to compile its own C++ code, which fails.
RUN apk add --no-cache \
    build-base \
    openssl-dev openssl-libs-static \
    pkgconfig \
    graphite2-dev graphite2-static \
    harfbuzz-dev harfbuzz-static \
    freetype-dev freetype-static \
    fontconfig-dev \
    icu-dev icu-static \
    zlib-static \
    libpng-static

# CRITICAL FIX: Tell Tectonic to use the system libraries we just installed
# instead of trying (and failing) to compile its own bundled C++ code.
ENV TECTONIC_DEP_BACKEND=pkg-config

# Allow pkg-config to run even though we are "cross-compiling" to musl
ENV PKG_CONFIG_ALLOW_CROSS=1

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

# Install runtime dependencies
# These shared libraries MUST match the -dev packages from the build stage
RUN apk add --no-cache \
    curl \
    libgcc \
    libstdc++ \
    graphite2 \
    harfbuzz \
    freetype \
    fontconfig \
    icu-data-full

# Creating a non root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Copy the binary files from builder stage
COPY --from=build --chown=appuser:appgroup /app/target/release/tex2pdf /usr/local/bin/app

# Switch to non root user
USER appuser

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:3000/healthz || exit 1

ENTRYPOINT ["/usr/local/bin/app"]