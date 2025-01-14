FROM rustlang/rust:nightly-alpine AS chef
RUN apk add musl-dev
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
RUN apk add openssl-libs-static openssl-dev
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY ./Cargo.toml ./Cargo.lock ./config_example.json ./
COPY ./src ./src
RUN cargo build --release
RUN mv ./target/release/rust-notifier ./program

FROM scratch AS runtime
WORKDIR /app
EXPOSE 3939
COPY --from=builder /app/program /app/config_example.json ./
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/app/program"]
