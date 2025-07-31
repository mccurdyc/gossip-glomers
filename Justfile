#!/usr/bin/env -S just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.
#
# Just Manual - https://just.systems/man/en/
# https://just.systems/man/en/working-directory.html
# Settings - https://just.systems/man/en/settings.html
# https://just.systems/man/en/settings.html#bash

set shell := ["/usr/bin/env", "bash", "-uc"]

log := "warn"
export JUST_LOG := log

set quiet := false

shebang := "/usr/bin/env bash"
docker_image := "coltonmccurdy/gossip"
maelstrom_cmd := 'java -Djava.awt.headless=true -jar "./maelstrom.jar" test'
maelstrom_test_cmd := maelstrom_cmd + " test"

# Variadic argument - https://just.systems/man/en/recipe-parameters.html
test *TEST:
    cargo test {{ TEST }} -- --nocapture

lint:
    cargo clippy

fmt:
    cargo fmt

docker-build:
    docker build \
      -f Dockerfile.dev \
      -t {{ docker_image }}:latest \
      .

docker-push: docker-build
    docker push {{ docker_image }}:latest

build challenge:
    cargo build --bin {{ challenge }} --release

maelstrom-run challenge:
    just build {{ challenge }}
    just "maelstrom-run-{{ challenge }}"

maelstrom-run-echo:
    {{ maelstrom_test_cmd }} \
      -w echo \
      --bin ./target/release/echo \
      --node-count 1 \
      --time-limit 10

maelstrom-run-unique:
    {{ maelstrom_test_cmd }} \
      -w unique-ids \
      --bin ./target/release/unique \
      --time-limit 30 \
      --rate 1000 \
      --node-count 3 \
      --availability total \
      --nemesis partition

maelstrom-run-broadcast:
    {{ maelstrom_test_cmd }} \
      -w broadcast \
      --bin ./target/release/broadcast \
      --node-count 1 \
      --time-limit 20 \
      --rate 10

maelstrom-run-broadcast-multi:
    {{ maelstrom_test_cmd }} \
      -w broadcast \
      --bin ./target/release/broadcast \
      --node-count 5 \
      --time-limit 20 \
      --rate 10

maelstrom-run-counter:
    {{ maelstrom_test_cmd }} \
      -w g-counter \
      --bin ./target/release/counter \
      --node-count 3 \
      --time-limit 20 \
      --rate 100 \
      --nemesis partition

maelstrom-run-replicated-log:
    {{ maelstrom_test_cmd }} \
      -w kafka \
      --bin ./target/release/replicated-log \
      --node-count 1 \
      --concurrency 2n \
      --time-limit 20 \
      --rate 100

maelstrom-serve:
    {{ maelstrom_cmd }} serve
