FROM rust:1.85.1-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
  pkg-config \
  libssl-dev

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-gnu --manifest-path "$PWD/twitch_chat_tracker/Cargo.toml"

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
  libssl-dev \
  pkg-config \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/twitch-chat-logger /app/
COPY --from=builder /app/config_files/deploy_config.yml /app/config_files/deploy_config.yml

WORKDIR /app

CMD ["./twitch-chat-logger"]
