# Builds the whole Rust workspace and ships the API, the migration tool and the
# CLI in the same image. Its build context is this directory, `backend/`.
#
#   docker build -t water-sort-api backend
#   docker run --rm -p 8080:8080 -e DATABASE_URL=postgres://... water-sort-api
#   docker run --rm -v "$PWD/backend/levels:/levels" water-sort-api water-sort /levels/level145.json

FROM rust:1-slim-bookworm AS builder

WORKDIR /build

# Warm the dependency cache on the manifests alone, so editing the sources does
# not rebuild the world.
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/format/Cargo.toml crates/format/Cargo.toml
COPY crates/cli/Cargo.toml crates/cli/Cargo.toml
COPY crates/api/Cargo.toml crates/api/Cargo.toml
COPY crates/migration/Cargo.toml crates/migration/Cargo.toml
RUN mkdir -p crates/core/src crates/format/src crates/cli/src crates/api/src crates/migration/src \
    && echo "" > crates/core/src/lib.rs \
    && echo "" > crates/format/src/lib.rs \
    && echo "fn main() {}" > crates/cli/src/main.rs \
    && echo "" > crates/api/src/lib.rs \
    && echo "fn main() {}" > crates/api/src/main.rs \
    && echo "" > crates/migration/src/lib.rs \
    && echo "fn main() {}" > crates/migration/src/main.rs \
    && cargo build --release --workspace --locked \
    && rm -rf crates

COPY crates ./crates
# Touch the real sources so cargo rebuilds them over the placeholder artifacts.
RUN find crates -name "*.rs" -exec touch {} + \
    && cargo build --release --workspace --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home water-sort

COPY --from=builder /build/target/release/water-sort-api /usr/local/bin/water-sort-api
COPY --from=builder /build/target/release/water-sort-migration /usr/local/bin/water-sort-migration
COPY --from=builder /build/target/release/water-sort /usr/local/bin/water-sort
COPY levels /opt/water-sort/levels

USER water-sort
WORKDIR /home/water-sort

ENV BIND_ADDRESS=0.0.0.0:8080 \
    RUST_LOG=water_sort_api=info,tower_http=info

EXPOSE 8080

CMD ["water-sort-api"]
