FROM clux/muslrust:stable AS chef
RUN cargo install cargo-chef
WORKDIR /vertretungsbot

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /vertretungsbot/recipe.json recipe.json
RUN apt update 
RUN apt install gcc-arm-linux-gnueabihf -y
RUN rustup target add armv7-unknown-linux-gnueabihf
RUN cargo chef cook --release --target armv7-unknown-linux-gnueabihf --recipe-path recipe.json
COPY . .
RUN cargo build --release --target armv7-unknown-linux-gnueabihf --bin vertretungsbot

FROM --platform=linux/arm32 arm32v7/debian:latest AS runtime
WORKDIR /vertretungsbot
RUN apt-get update && \
    apt-get install ca-certificates -y && \
    apt-get clean && \
    update-ca-certificates
COPY --from=builder /vertretungsbot/target/release/vertretungsbot /usr/bin/
CMD [ "/usr/bin/vertretungsbot" ] 