FROM rust:1.89-slim AS builder

RUN USER=root cargo new --bin thay
WORKDIR /thay

COPY ./Cargo.lock ./Cargo.toml ./
RUN cargo build --release && \
    rm src/*.rs && \
    rm ./target/release/deps/thay*

COPY ./src ./src
COPY ./migrations ./migrations
RUN cargo build --release

####################################

FROM debian:stable-slim AS runner

COPY --from=builder /thay/target/release/thay /usr/src/thay
ENTRYPOINT ["/usr/src/thay"]
