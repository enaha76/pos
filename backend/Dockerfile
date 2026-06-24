# ---- build stage ----
FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache dependencies: build a dummy main against the manifest first.
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Now build the real sources (migrations are embedded at compile time).
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# ---- runtime stage ----
# debian-slim from Docker Hub (distroless/cc on gcr.io is unreachable here).
# For an even smaller (~20MB) hardened image where gcr.io is reachable, swap to
# gcr.io/distroless/cc-debian12.
FROM debian:bookworm-slim AS runtime
# Copy the CA bundle from the builder (it's debian-based) instead of an apt
# download — avoids a slow network step and keeps the runtime image offline-buildable.
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/target/release/cashier-charge-api /usr/local/bin/api
EXPOSE 8080
ENV PORT=8080
CMD ["/usr/local/bin/api"]
