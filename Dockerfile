FROM rust:1.78-bookworm AS builder
WORKDIR /workspace
ENV CARGO_HOME=/workspace/.cargo-home
ENV CARGO_TARGET_DIR=/workspace/target
COPY . .
RUN cargo build --release --bin proofgate-gateway

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /workspace/target/release/proofgate-gateway /usr/local/bin/proofgate-gateway
COPY config /app/config
EXPOSE 8080
ENTRYPOINT ["proofgate-gateway"]

