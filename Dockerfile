FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libpq-dev curl && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

RUN cargo install sqlx-cli --no-default-features --features postgres

COPY . .

RUN touch src/main.rs && \
    cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 libpq5 curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /app/target/release/templates-service /app/templates-service
COPY migrations /app/migrations

EXPOSE 8080

CMD /bin/bash -c "sqlx migrate run && ./templates-service"