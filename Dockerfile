# FROM clux/muslrust:stable AS chef
# RUN cargo install cargo-chef
# #RUN dpkg --add-architecture arm
# RUN rustup target add armv7-unknown-linux-musleabihf
# RUN apt-get update && apt-get -y install binutils-arm-linux-gnueabihf gcc-arm-linux-gnueabihf 
# #RUN apt-get -y install libssl-dev
# WORKDIR /vertretungsbot

# FROM chef AS planner
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json

# FROM chef AS builder
# COPY --from=planner /vertretungsbot/recipe.json recipe.json
# RUN cargo chef cook --release --target armv7-unknown-linux-musleabihf --recipe-path recipe.json
# COPY . .
# RUN cargo build --release --target armv7-unknown-linux-musleabihf --bin vertretungsbot

FROM --platform=linux/arm32 arm32v7/debian:latest 
WORKDIR /vertretungsbot
RUN apt-get update && \
    apt-get install ca-certificates -y && \
    apt-get clean && \
    update-ca-certificates
#COPY --from=builder /vertretungsbot/target/release/vertretungsbot /usr/bin/
CMD [ "echo 'Fuck u" ] 