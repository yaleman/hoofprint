[private]
default:
    @just --list

test:
    cargo test --quiet

lint:
    cargo clippy --quiet --all-targets --all-features

check: test lint