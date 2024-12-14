#!/usr/bin/env -S just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

log := "warn"
export JUST_LOG := log

# Variadic argument - https://just.systems/man/en/recipe-parameters.html
test *TEST:
    cargo test {{ TEST }} -- --nocapture

lint:
    cargo clippy

build challenge:
    cargo build --bin {{ challenge }} --release

maelstrom-run challenge:
    just "maelstrom-run-{{ challenge }}"

maelstrom-run-broadcast:
    just build broadcast
    java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w broadcast --bin ./target/release/broadcast --node-count 1 --time-limit 20 --rate 10

maelstrom-serve:
    java -Djava.awt.headless=true -jar "./maelstrom.jar" serve
