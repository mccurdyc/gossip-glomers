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
