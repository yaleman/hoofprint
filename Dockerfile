FROM rust:1.92.0-slim-trixie AS builder

ARG GITHUB_SHA="$(git rev-parse HEAD)"
LABEL com.hoofprint.git-commit="${GITHUB_SHA}"


# fixing the issue with getting OOMKilled in BuildKit
RUN mkdir /hoofprint
COPY . /hoofprint/

WORKDIR /hoofprint
# install the dependencies
RUN apt-get update && apt-get -q install -y \
    git \
    clang \
    pkg-config \
    mold
ENV CC="/usr/bin/clang"
RUN cargo build --quiet --release --bin hoofprint
RUN chmod +x /hoofprint/target/release/hoofprint

# FROM gcr.io/distroless/cc-debian12 AS hoofprint
FROM rust:1.92.0-slim-trixie AS secondary

RUN apt-get -y remove --allow-remove-essential \
    binutils cpp cpp-14 gcc gcc grep gzip ncurses-bin ncurses-base sed && apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/* && rm -rf /usr/local/cargo /usr/local/rustup

# # ======================
# https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile
COPY --from=builder /hoofprint/target/release/hoofprint /
WORKDIR /
RUN useradd -m nonroot

FROM scratch AS final
ARG DESCRIPTION="hoofPrint"
ARG GITHUB_SHA="unknown"
LABEL DESCRIPTION="${DESCRIPTION}"
LABEL com.hoofprint.git-commit="${GITHUB_SHA}"

COPY --from=secondary / /
ADD ./static /static

ENV HOOFPRINT_DB_FILE="/data/hoofprint.sqlite"
ENV HOOFPRINT_HOST="0.0.0.0"

USER nonroot
ENTRYPOINT ["./hoofprint"]


