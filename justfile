[private]
default:
    @just --list

test:
    cargo test --quiet

lint:
    cargo clippy --quiet --all-targets --all-features

check: test lint


docker_build:
    docker build -t ghcr.io/yaleman/hoofprint:latest .

docker_run:
    docker run --rm -it -p 3000:3000 \
        --mount "type=bind,src=$(pwd)/hoofprint.sqlite,target=/data/hoofprint.sqlite" \
        ghcr.io/yaleman/hoofprint:latest