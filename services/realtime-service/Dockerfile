FROM rust:1.88-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY services/ services/

# Build the service
RUN cargo build --release -p realtime-service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/realtime-service /usr/local/bin/realtime-service

EXPOSE 3006

CMD ["realtime-service"]
