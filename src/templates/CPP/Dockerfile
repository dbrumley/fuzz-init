FROM debian:stable-slim

RUN apt-get update && apt-get install -y \
    build-essential clang lld make cmake git curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project
WORKDIR /app
COPY . /app

{{#unless minimal}}
# Build the main application library first
RUN make lib

# Run unit tests to verify library functionality
RUN make test

# Build the fuzzer using the appropriate target
RUN make fuzz-{{default_fuzzer}}
{{else}}
# Minimal mode - just build the fuzzer
RUN cd fuzz && make
{{/unless}}