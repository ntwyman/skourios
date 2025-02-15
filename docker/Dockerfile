#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update -q && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    # for seL4
    gcc-aarch64-linux-gnu \
    gdb \
    python3-dev \
    python3-venv \
    cmake \
    ninja-build \
    device-tree-compiler \
    libxml2-utils \
    qemu-utils \
    qemu-system-arm \
    qemu-efi-aarch64 \
    ipxe-qemu \
    # for bindgen
    libclang-dev \
    # for test script
    python3-pexpect \
    # for hacking
    bash-completion \
    man \
    sudo \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /tmp

RUN curl -sSf https://sh.rustup.rs | \
    bash -s -- -y --no-modify-path \
    --default-toolchain nightly-2024-09-01 \
    --component rust-src

ENV PATH=/root/.cargo/bin:$PATH

ENV SEL4_INSTALL_DIR=/opt/seL4

RUN set -eux; \
    git clone \
    https://github.com/seL4/seL4.git \
    --config advice.detachedHead=false; \
    cd seL4; \
    git checkout cd6d3b8c25d49be2b100b0608cf0613483a6fffa;

RUN set -eux; \
    cd seL4; \
    python3 -m venv pyenv; \
    export PATH=$(realpath ./pyenv/bin):$PATH; \
    pip install tools/python-deps; \
    cmake \
    -DCROSS_COMPILER_PREFIX=aarch64-linux-gnu- \
    -DCMAKE_INSTALL_PREFIX=$SEL4_INSTALL_DIR \
    -DKernelPlatform=qemu-arm-virt \
    -DKernelArmHypervisorSupport=ON \
    -DKernelVerificationBuild=OFF \
    -DARM_CPU=cortex-a57 \
    -G Ninja \
    -S . \
    -B build; \
    ninja -C build all; \
    ninja -C build install; \
    rm -rf $(pwd);

RUN set -eux; \
    url="https://github.com/seL4/rust-sel4"; \
    rev="1cd063a0f69b2d2045bfa224a36c9341619f0e9b"; \
    common_args="--git $url --rev $rev --root $SEL4_INSTALL_DIR"; \
    CC_aarch64_unknown_none=aarch64-linux-gnu-gcc \
    SEL4_PREFIX=$SEL4_INSTALL_DIR \
    cargo install \
    -Z build-std=core,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    --target aarch64-unknown-none \
    $common_args \
    sel4-kernel-loader; \
    cargo install \
    $common_args \
    sel4-kernel-loader-add-payload;

ARG UNAME
RUN adduser $UNAME
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
USER $UNAME

RUN set -eux; \
    curl -sSf https://sh.rustup.rs | \
    bash -s -- -y --no-modify-path --default-toolchain none;

ENV PATH=/home/$UNAME/.cargo/bin:$PATH

ENV SHELL=/bin/bash

WORKDIR /work
