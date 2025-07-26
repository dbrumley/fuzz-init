# Use Ubuntu 22.04 as base
FROM mcr.microsoft.com/vscode/devcontainers/base:debian

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install claude code. 
RUN apt-get update && apt-get install -y \
    nodejs \
    npm

RUN npm install -g @anthropic-ai/claude-code


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
    clang \
    afl++ 

#\
#&& rm -rf /var/lib/apt/lists/*



# Install Rust
USER vscode
RUN curl  https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Install cargo-fuzz for Rust fuzzing
RUN /home/vscode/.cargo/bin/cargo install cargo-fuzz


# Switch back to root for system installations
USER root


# Install HonggFuzz
# Hongfuzz dependencies 
RUN apt-get install -y \
    pkg-config \ 
    libunwind8-dev \ 
    binutils-dev 

RUN cd /tmp && \
    git clone https://github.com/google/honggfuzz.git && \
    cd honggfuzz && \
    make && \
    make install && \
    cd / && rm -rf /tmp/honggfuzz


# Set up environment for AFL
ENV AFL_SKIP_CPUFREQ=1
ENV AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1

# Install additional Python packages for fuzzing
# TODO: Fix so atheris can be installed
#RUN pip3 install atheris

# Switch back to vscode user
USER vscode

# Set working directory
WORKDIR /workspace

# Add welcome message to bashrc
RUN echo 'echo "🔧 Fuzzing Environment Ready! Run: cargo run -- --test"' >> ~/.bashrc
RUN echo 'echo "🤖 Claude Code available! Run: claude --help"' >> ~/.bashrc

# Default command
CMD ["/bin/bash"]