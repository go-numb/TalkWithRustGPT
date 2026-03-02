FROM node:20-bookworm

ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /tools

# Build tools + mingw (Windows GNU cross) + helpers
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl git xz-utils \
    build-essential pkg-config \
    mingw-w64 \
    rsync \
  && rm -rf /var/lib/apt/lists/*

# Rust (rustup) + Windows target
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal \
 && /opt/cargo/bin/rustup toolchain install stable \
 && /opt/cargo/bin/rustup default stable \
 && /opt/cargo/bin/rustup target add x86_64-pc-windows-gnu

# Cross compiler env for cargo (GNU)
ENV CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc \
    CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++ \
    AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar \
    WINDRES_x86_64_pc_windows_gnu=x86_64-w64-mingw32-windres

WORKDIR /runner
COPY docker/win-cross/entrypoint.sh /runner/entrypoint.sh
RUN chmod +x /runner/entrypoint.sh

# /src (read-only mount) -> copied into /build inside container
# /out (write mount) -> artifacts
ENTRYPOINT ["/runner/entrypoint.sh"]
