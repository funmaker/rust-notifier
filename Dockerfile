FROM rustlang/rust:nightly-alpine as chef
RUN apk add musl-dev
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
ARG CRATE_NAME
RUN test -n "$CRATE_NAME" || (echo -e "\n\033[0;31mCRATE_NAME build-arg must be set\033[0m\n" && false)
RUN apk add openssl-libs-static openssl-dev
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY ./Cargo.toml ./Cargo.lock ./config_example.json ./
COPY ./src ./src
RUN cargo build --release
RUN mv ./target/release/${CRATE_NAME} ./program

FROM scratch AS runtime
WORKDIR /app
ENV CRATE_NAME=$CRATE_NAME
COPY --from=builder /app/program /app/config_example.json ./
ENTRYPOINT ["/app/program"]
