FROM rust:1.95-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential libdbus-1-dev pkg-config protobuf-compiler \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
RUN cd rust && cargo build --locked --release -p revault_tooling
RUN install -m 0755 rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN cargo check --locked --manifest-path bindings/rust/Cargo.toml
RUN cargo build --locked --release --manifest-path bindings/e2e/rust/Cargo.toml \
    && install -m 0755 bindings/e2e/rust/target/release/revault-rust-conformance \
      /opt/revault-rust-conformance
ENV REVAULT_E2E_LANGUAGE=rust
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
CMD ["revault-tool", "e2e", "container", "--language", "rust"]
