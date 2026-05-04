# syntax=docker/dockerfile:1

FROM rust:1.85-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        clang \
        cmake \
        libasound2-dev \
        libgtk-3-dev \
        libssl-dev \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libxkbcommon-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY README.md readme-cn.md LICENSE ./

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

ARG APP_VERSION=dev

LABEL org.opencontainers.image.title="AI-Movie-Player" \
      org.opencontainers.image.description="AI-native local movie library companion" \
      org.opencontainers.image.url="https://github.com/peixl/AI-Movie-Player" \
      org.opencontainers.image.source="https://github.com/peixl/AI-Movie-Player" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${APP_VERSION}"

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libasound2 \
        libgtk-3-0 \
        libssl3 \
        libxcb-render0 \
        libxcb-shape0 \
        libxcb-xfixes0 \
        libxkbcommon0 \
        xdg-utils \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ai-movie-player /usr/local/bin/AI-Movie-Player
COPY --from=builder /app/README.md /opt/ai-movie-player/README.md
COPY --from=builder /app/readme-cn.md /opt/ai-movie-player/readme-cn.md
COPY --from=builder /app/LICENSE /opt/ai-movie-player/LICENSE

ENTRYPOINT ["AI-Movie-Player"]