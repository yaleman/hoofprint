[private]
default:
    @just --list

test:
    cargo test --quiet

# Check the JavaScript and CSS files using Biome
js-lint:
    biome check static/*.css static/*.js

rust-lint:
    cargo clippy --quiet --all-targets --all-features

lint: rust-lint js-lint

check: test lint


docker_build:
    docker build -t ghcr.io/yaleman/hoofprint:latest .

docker_run:
    docker run --rm -it -p 3000:3000 \
        --mount "type=bind,src=$(pwd)/hoofprint.sqlite,target=/data/hoofprint.sqlite" \
        ghcr.io/yaleman/hoofprint:latest

coverage:
    cargo tarpaulin --coveralls ${COVERALLS_REPO_TOKEN} \
        --exclude-files src/main.rs \
        --exclude-files src/cli.rs
    @echo "Coveralls repo information is at https://coveralls.io/github/yaleman/hoofprint"