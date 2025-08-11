FROM debian:stable-slim

RUN apt-get update && apt-get install -y \
    build-essential clang lld make cmake git curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project
WORKDIR /app
COPY . /app

{{#unless minimal}}
# Build the main application library first
RUN ./fuzz.sh build
{{else}}
# Minimal mode - just build the fuzzer
RUN cd fuzz && ./fuzz.sh build 
{{/unless}}