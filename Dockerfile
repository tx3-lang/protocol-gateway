# ---- Build stage ----
FROM rust:1.93-slim AS builder

WORKDIR /build

# Cache dependencies: build with a dummy main first so source changes
# don't invalidate the dependency layer.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --create-home appuser

WORKDIR /app

COPY --from=builder /build/target/release/api-layer-tx3-protocols /usr/local/bin/api-layer-tx3-protocols
COPY protocols ./protocols

USER appuser

ENV PORT=8080 \
    NETWORK=mainnet \
    PROTOCOLS_DIR=/app/protocols

EXPOSE 8080

CMD ["api-layer-tx3-protocols"]
