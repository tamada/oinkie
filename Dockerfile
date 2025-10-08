FROM rust:1-bookworm as builder

# 必要なツールをインストール
RUN apt-get update && apt-get install -y \
    wget  \
    gnupg \
    software-properties-common \
    lsb-release \
    curl

# LLVM の公式リポジトリを追加
RUN bash -c "$(wget -O - https://apt.llvm.org/llvm.sh)"

RUN apt-get install -y llvm-19 clang-19 lldb-19 lld-19 libpolly-19-dev

# デフォルトの clang を clang-19 に設定（任意）
RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-19 100 && \
    update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-19 100

COPY . /app
RUN  cd /app && cargo build --release

FROM debian:bookworm-slim as runner

RUN adduser --home /app nonroot

COPY --from=builder /app/target/release/oinkie-cli     /opt/oinkie/oinkie
COPY --from=builder /app/target/release/liboinkie.rlib /opt/oinkie/liboinkie.rlib

ENV PATH="/opt/oinkie:${PATH}"

WORKDIR /app
USER    nonroot
