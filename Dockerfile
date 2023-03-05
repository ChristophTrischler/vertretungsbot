# FROM clux/muslrust:stable AS chef
# RUN cargo install cargo-chef
# #RUN 
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

FROM joaonortech/rust-multiarch:build-armv7-from-amd64
RUN cargo install cargo-chef
RUN echo "deb http:// bionic main " >> /etc/apt/sources.list
RUN dpkg --add-architecture armhf
RUN apt-get update && apt-get -y install openssl libssl-dev:armhf
WORKDIR /vertretungsbot
COPY . . 

# FROM chef AS planner
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json

# FROM chef AS builder
#COPY --from=planner /vertretungsbot/recipe.json recipe.json
#RUN cargo chef cook --release --recipe-path recipe.json
#COPY . .
# RUN cargo build --release  --bin vertretungsbot

# FROM --platform=linux/arm32 arm32v7/debian:latest as runtime
# WORKDIR /vertretungsbot
# RUN apt-get update && \
#     apt-get install ca-certificates -y && \
#     apt-get clean && \
#     update-ca-certificates
# COPY --from=builder /vertretungsbot/target/release/vertretungsbot /usr/bin/
# CMD [ "vertretungsbot" ] 