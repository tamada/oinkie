# One build, two images: `full-image`, which carries Ghidra, and `light-image`,
# which does not. They share the builder stage below, and that sharing is the
# point of the file: when this was two files with identical builder halves, a
# directory that moved had to be fixed in both, and was fixed in neither (#91).
#
# `docker build` with no --target builds the *last* stage, which is
# `light-image`. Anything that wants the other must say so.

ARG GIT_REVISION
ARG BUILD_DATE
ARG VERSION
ARG AUTHOR="Haruaki TAMADA"
ARG TITLE="oinkie"
ARG URL="https://tamada.github.io/oinkie"
ARG SOURCE="https://github.com/tamada/oinkie"
ARG LICENSE="MIT"

# ==========================================
# Stage 1: Build the Rust oinkie CLI binary
# ==========================================
FROM rust:1.95-slim-bookworm AS builder

# Install system dependencies needed for compiling
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/oinkie
COPY Cargo.toml .
COPY Cargo.lock .
COPY src        ./src
COPY cli        ./cli
# The lifting script is embedded into the binary with `include_str!`, so it
# has to be in the build context. `assets/lifters` rather than all of
# `assets`, which also holds shell completions and result-summarising
# scripts that the build does not read.
COPY assets/lifters ./assets/lifters

# Build the release binary.
#
# --features mcp, once, for both images: it is the same binary either way, a
# few megabytes larger, and building it for only one of them would mean
# rebuilding the other on the day lifting becomes an MCP tool. Which of the two
# an MCP client should be pointed at is a question about the runtime stages
# below, not about this one.
RUN cargo build --release --locked --features mcp

# ==========================================
# Stage 2: Runtime environment with JDK & Ghidra
# ==========================================
FROM eclipse-temurin:21-jdk-noble AS full-image

ARG GIT_REVISION
ARG BUILD_DATE
ARG VERSION
ARG AUTHOR
ARG TITLE
ARG URL
ARG SOURCE
ARG LICENSE

LABEL org.opencontainers.image.authors=${AUTHOR} \
      org.opencontainers.image.title=${TITLE} \
      org.opencontainers.image.description="oinkie with the Ghidra lifter included. Detects software theft by comparing birthmarks extracted from binaries." \
      org.opencontainers.image.url=${URL} \
      org.opencontainers.image.source=${SOURCE} \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.revision=${GIT_REVISION} \
      org.opencontainers.image.created=${BUILD_DATE} \
      org.opencontainers.image.licenses=${LICENSE}

# Install system utilities needed by Ghidra and oinkie
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    unzip \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Ghidra
# You can customize the Ghidra version using build arguments
ARG GHIDRA_VERSION=11.0.3
ARG GHIDRA_BUILD_DATE=20240410
ARG GHIDRA_ZIP_NAME=ghidra_${GHIDRA_VERSION}_PUBLIC_${GHIDRA_BUILD_DATE}.zip
ARG GHIDRA_DOWNLOAD_URL=https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_${GHIDRA_VERSION}_build/${GHIDRA_ZIP_NAME}
ARG GHIDRA_SHA256=2462a2d0ab11e30f9e907cd3b4aa6b48dd2642f325617e3d922c28e752be6761

ENV GHIDRA_HOME=/opt/ghidra

RUN  wget --quiet "${GHIDRA_DOWNLOAD_URL}" -O /tmp/ghidra.zip \
  && if [ -n "${GHIDRA_SHA256}" ]; then echo "${GHIDRA_SHA256}  /tmp/ghidra.zip" | sha256sum -c -; fi \
  && unzip -q /tmp/ghidra.zip -d /opt \
  && ln -s "/opt/ghidra_${GHIDRA_VERSION}_PUBLIC" "${GHIDRA_HOME}" \
  && rm /tmp/ghidra.zip

# Copy the compiled oinkie binary from the builder stage
COPY --from=builder /usr/src/oinkie/target/release/oinkie /usr/local/bin/oinkie

# Set environment and working directory
WORKDIR /work
ENV PATH="/usr/local/bin:${PATH}"

# Default command
ENTRYPOINT ["oinkie"]
CMD ["--help"]

# ==========================================
# Stage 3: Tiny runtime environment for CLI execution (Default)
# ==========================================
FROM debian:bookworm-slim AS light-image

ARG GIT_REVISION
ARG BUILD_DATE
ARG VERSION
ARG AUTHOR
ARG TITLE
ARG URL
ARG SOURCE
ARG LICENSE

LABEL org.opencontainers.image.authors=${AUTHOR} \
      org.opencontainers.image.title=${TITLE} \
      org.opencontainers.image.description="oinkie without Ghidra, for programs that are already lifted. Detects software theft by comparing birthmarks extracted from binaries." \
      org.opencontainers.image.url=${URL} \
      org.opencontainers.image.source=${SOURCE} \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.revision=${GIT_REVISION} \
      org.opencontainers.image.created=${BUILD_DATE} \
      org.opencontainers.image.licenses=${LICENSE}

# Copy the compiled oinkie binary from the builder stage
COPY --from=builder /usr/src/oinkie/target/release/oinkie /usr/local/bin/oinkie

# Set environment and working directory
WORKDIR /work
ENV PATH="/usr/local/bin:${PATH}"

# Default command
ENTRYPOINT ["oinkie"]
CMD ["--help"]
