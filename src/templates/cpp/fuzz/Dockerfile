FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    clang lld make cmake git curl \
    && rm -rf /var/lib/apt/lists/*

COPY . /fuzz
WORKDIR /fuzz

RUN chmod +x build.sh && ./build.sh
