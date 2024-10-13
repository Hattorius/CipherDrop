# Building
FROM rust:1.81 as builder

WORKDIR /usr/src/cipherdrop

COPY backend/ backend/
COPY frontend/ frontend/

WORKDIR /usr/src/cipherdrop/backend

RUN cargo build --release


# Image
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/cipherdrop/backend/target/release/backend /usr/local/bin/backend

COPY --from=builder /usr/src/cipherdrop/backend /app/backend
COPY --from=builder /usr/src/cipherdrop/frontend /app/frontend

WORKDIR /app/backend

ENTRYPOINT ["backend"]
