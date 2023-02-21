FROM rust:1 as builder

RUN rustup update nightly; rustup default nightly;

COPY ./ ./

RUN cargo build --release

# Run the binary
CMD ["./target/release/test-sendfiles"] 