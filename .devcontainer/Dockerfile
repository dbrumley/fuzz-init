# Use Ubuntu 22.04 as base
FROM mcr.microsoft.com/vscode/devcontainers/base:ubuntu-22.04

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install essential packages
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    curl \
    wget \
    cmake \
    ninja-build \
    pkg-config \
    python3 \
    python3-pip \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install specific clang version to avoid conflicts
RUN apt-get update && apt-get install -y \
    clang-14 \
    clang++-14 \
    llvm-14 \
    libc++-14-dev \
    libc++abi-14-dev \
    && rm -rf /var/lib/apt/lists/*

# Create symlinks for default clang commands
RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-14 100 && \
    update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-14 100 && \
    update-alternatives --install /usr/bin/llvm-config llvm-config /usr/bin/llvm-config-14 100

# Install Rust
USER vscode
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Install cargo-fuzz for Rust fuzzing
RUN /home/vscode/.cargo/bin/cargo install cargo-fuzz

# Switch back to root for system installations
USER root

# Install AFL++
RUN cd /tmp && \
    git clone https://github.com/AFLplusplus/AFLplusplus.git && \
    cd AFLplusplus && \
    make distrib && \
    make install && \
    cd / && rm -rf /tmp/AFLplusplus

# Install HonggFuzz
RUN cd /tmp && \
    git clone https://github.com/google/honggfuzz.git && \
    cd honggfuzz && \
    make && \
    make install && \
    cd / && rm -rf /tmp/honggfuzz

# Create symlinks for common AFL commands to ensure they're in PATH
RUN ln -sf /usr/local/bin/afl-clang /usr/bin/afl-clang && \
    ln -sf /usr/local/bin/afl-clang++ /usr/bin/afl-clang++ && \
    ln -sf /usr/local/bin/afl-fuzz /usr/bin/afl-fuzz

# Set up environment for AFL
ENV AFL_SKIP_CPUFREQ=1
ENV AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1

# Install additional Python packages for fuzzing
RUN pip3 install atheris

# Switch back to vscode user
USER vscode

# Set working directory
WORKDIR /workspace

# Add welcome message to bashrc
RUN echo 'echo "🔧 Fuzzing Environment Ready! Run: cargo run -- --test"' >> ~/.bashrc

# Default command
CMD ["/bin/bash"]