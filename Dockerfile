FROM lukemathwalker/cargo-chef:0.1.67-rust-1.80-alpine3.20 as chef
WORKDIR /usr/src/app

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
WORKDIR /usr/src/app
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
# Build our project
COPY . .
RUN cargo build --release

FROM alpine:3.20 AS runtime
RUN apk add --no-cache openssl ca-certificates
COPY --from=builder /usr/src/app/target/release/ /usr/local/bin/
CMD ["/usr/local/bin/echo"]
