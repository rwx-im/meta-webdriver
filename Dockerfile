FROM rust:1.59.0 as deps

WORKDIR /usr/src

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo 'fn main {}' > src/main.rs
RUN cargo fetch

FROM rust:1.59.0 as builder

RUN apt update \
  && apt install -y libssl-dev

WORKDIR /usr/src

COPY --from=deps /usr/local/cargo /usr/local/cargo
COPY Cargo.toml Cargo.lock ./
COPY src ./src/

RUN cargo build --release
RUN strip --strip-unneeded target/release/meta-webdriver

FROM debian:11.2-slim

LABEL org.opencontainers.image.authors="Mikkel Kroman <mk@maero.dk>"

COPY --from=builder /usr/src/target/release/meta-webdriver /usr/local/bin/meta-webdriver

RUN apt update \
  && apt install -y chromium-driver xvfb

RUN adduser --system --home /workspace usr
WORKDIR /workspace
USER usr

EXPOSE 3000

CMD ["/bin/sh", "-c", "xvfb-run /usr/local/bin/meta-webdriver"]
