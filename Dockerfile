FROM --platform=linux/amd64 lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /vertretungsbot

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /vertretungsbot/recipe.json recipe.json
RUN rustup update nightly; rustup default nightly;
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin vertretungsbot

# We do not need the Rust toolchain to run the binary!
FROM debian:stable-slim AS runtime
WORKDIR /vertretungsbot
RUN apt-get update && \ 
    apt-get install ca-certificates -y && \
    apt-get clean && \
    update-ca-certificates
COPY --from=builder /vertretungsbot/target/release/vertretungsbot /usr/bin/
COPY .env .
CMD [ "vertretungsbot" ] 