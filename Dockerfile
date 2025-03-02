FROM rust:latest as builder

WORKDIR /workspace

COPY . .

RUN cargo build --release;
RUN mkdir /workspace/app_log

VOLUME ./workspace/app_log
CMD ["./target/release/twitch-chat-logger"]

# # COPY app_config/ app/app_config
# # COPY database_connection/ app/database_connection
# # COPY entities/ app/entities
# # COPY src/ app/src
# # COPY Cargo.toml app/Cargo.toml
#
# # Builder
# WORKDIR /workspace/app
#
# RUN ls -R;
# RUN cargo build --release;
#
# # Runtime
# FROM debian:bookworm-slim
#
# WORKDIR /workspace/app
#
# RUN ls -R && sleep infinity;
# COPY --from=builder target/release/twitch-chat-logger .
#
# CMD ["./target/release/app"]
#
#------------------------------------------------------------------------------
#
# FROM debian:bullseye-slim
#
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
#
# COPY --from=builder /usr/local/cargo/bin/app /usr/local/bin/app
#
# CMD ["app"]
#
