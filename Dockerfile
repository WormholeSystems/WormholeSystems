# The Rust API (Axum). Migrations are embedded; the SDE seed data is bundled so a fresh
# database seeds itself on first boot.
# Pinned to bookworm to match the runtime stage below: the default slim tag tracks a newer
# Debian, and a binary linked against its glibc will not start on bookworm.
FROM rust:1.94-slim-bookworm AS chef
# cargo-chef splits the build in two: the dependencies, which change once in a while, and
# this crate, which changes every commit. The first half is its own layer, so a rebuild
# after a code change compiles a few hundred files of ours instead of every crate again.
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /app

# Only the manifests matter to the recipe, but cargo-chef reads them out of a full tree.
FROM chef AS plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=plan /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock sqlx.toml ./
# The workspace names it, so cargo needs it present even though `default-members` keeps it
# out of this build. Its manifest and sources are a few KB.
COPY wsctl ./wsctl
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    cargo build --release

FROM debian:bookworm-slim
# curl is for the compose healthcheck, which asks the API over HTTP whether it is serving.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/wormholesystems /usr/local/bin/wormholesystems
COPY data ./data
ENV LISTEN_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["wormholesystems"]
