# Use Debian as the base. 
FROM debian:stable

# Use this instead if you want a devcontainer for vscode
#FROM mcr.microsoft.com/vscode/devcontainers/base:debian

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive


# Install core packages. Remove any fuzzers you do not need.
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


# Needed for hongfuzz: install dependencies and honggfuzz
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


# If using vscode, switch to that user. 
USER vscode

# Set working directory
WORKDIR /workspace

# Add welcome message to bashrc
RUN echo 'echo "🔧 Fuzzing Environment Ready!"' >> ~/.bashrc

# Copy in code
COPY . .

# Build fuzzing targets
RUN ./fuzz.sh build

# Uncomment to run a short test
#RUN ./fuzz.sh test

# Default command
CMD ["/bin/bash"]