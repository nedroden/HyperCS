# syntax=docker/dockerfile:1
FROM rust:1 AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
# Build dependencies first for better layer caching
RUN mkdir src && echo 'fn main() {}' > src/main.rs && touch src/lib.rs \
    && cargo build --release && rm -rf src
COPY migrations ./migrations
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release

FROM debian:stable-slim
RUN useradd --system --no-create-home hypercs
COPY --from=build /app/target/release/hypercs-api /usr/local/bin/hypercs-api
USER hypercs
EXPOSE 8080
CMD ["hypercs-api"]
