FROM rust:1.72 as builder
WORKDIR /usr/src/server
COPY . .
RUN cargo test
RUN cargo install --path .

FROM debian:stable-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
CMD ["server"]
