FROM appimagecrafters/appimage-builder:latest@sha256:0188548ac837d832666bcaeb6a7833d575eba8ef1fedb6cd7717a74775dce2c9

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential ca-certificates curl pkg-config \
    libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev libx11-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.1
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /src
