FROM lukemathwalker/cargo-chef:0.1.67-rust-1.80-alpine3.20 as pin
WORKDIR /app
RUN apk update \
  && apk add --no-cache musl-utils clang

FROM pin as hash
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM pin as builder
WORKDIR /app
COPY --from=hash /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
# Build our project
# Need to cross-compile for alpine
RUN cargo build --target x86_64-unknown-linux-musl --release --bin echo

FROM alpine:3.20 AS runtime
WORKDIR /app
RUN apk update \
  && apk add openssl ca-certificates \
  && apk cache clean
COPY --from=builder /app/target/release/echo echo
ENTRYPOINT ["echo"]
