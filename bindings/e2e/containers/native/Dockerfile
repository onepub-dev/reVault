FROM rust:1.95-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    libdbus-1-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY rust rust
COPY bindings/flatbuffers bindings/flatbuffers
COPY bindings/release bindings/release
COPY bindings/ruby/native/revault_ruby_shim.c bindings/ruby/native/revault_ruby_shim.c
RUN cd rust && cargo build --locked --release -p revault_bindings -p revault_tooling
RUN install -m 0755 rust/target/release/revault-tool /usr/local/bin/revault-tool \
    && revault-tool release package-native --repository . \
      --version 0.2.0 --target linux-x86_64-gnu \
      --library rust/target/release/librevault_api.so \
      --static-library rust/target/release/librevault_api.a --output /tmp/native
