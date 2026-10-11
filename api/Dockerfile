# syntax=docker/dockerfile:1
FROM rust:1 AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/domain/Cargo.toml crates/domain/Cargo.toml
COPY crates/db/Cargo.toml crates/db/Cargo.toml
COPY crates/api/Cargo.toml crates/api/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml
# Build dependencies first for better layer caching (queries are runtime-checked,
# so this - and the whole image - builds without a live database).
RUN mkdir -p crates/domain/src crates/db/src crates/api/src crates/server/src \
    && echo 'fn main() {}' > crates/server/src/main.rs \
    && touch crates/domain/src/lib.rs crates/db/src/lib.rs crates/api/src/lib.rs crates/server/src/lib.rs \
    && cargo build --release --workspace \
    && rm -rf crates/*/src
COPY crates/db/migrations crates/db/migrations
COPY crates/domain/src crates/domain/src
COPY crates/db/src crates/db/src
COPY crates/api/src crates/api/src
COPY crates/server/src crates/server/src
RUN find crates -name '*.rs' -exec touch {} + && cargo build --release --workspace

FROM debian:stable-slim
RUN useradd --system --no-create-home hypercs
COPY --from=build /app/target/release/hypercs-server /usr/local/bin/hypercs-server
USER hypercs
EXPOSE 8080
CMD ["hypercs-server"]
