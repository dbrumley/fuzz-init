FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    clang lld make cmake git curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project
COPY . /app
WORKDIR /app

{{#unless minimal}}
# Build the main application library first
RUN make lib

# Build the fuzzer using the appropriate target
RUN make fuzz-{{default_fuzzer}}
{{else}}
# Minimal mode - just build the fuzzer
RUN cd fuzz && make
{{/unless}}