FROM rust:1.95-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-dev pkg-config \
    protobuf-compiler python3 python3-pip python3-protobuf \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .
RUN cd rust && cargo build --locked --release -p revault_bindings -p revault_tooling
RUN install -m 0755 rust/target/release/revault-tool /usr/local/bin/revault-tool \
    && revault-tool release package-native --repository . \
      --version 0.2.0 --target linux-x86_64-gnu \
      --library rust/target/release/librevault_api.so \
      --static-library rust/target/release/librevault_api.a --output /tmp/native \
    && revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.2.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.2.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN python3 -m pip install --break-system-packages /tmp/packages/python/linux-x86_64-gnu

ENV REVAULT_E2E_INSTALLED=1
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/lib/python3.11/dist-packages/revault_api/_native/linux-x86_64-gnu
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "python"]
