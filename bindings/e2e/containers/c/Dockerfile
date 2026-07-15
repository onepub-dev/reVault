FROM rust:1.95-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential cmake dbus-x11 gnome-keyring libsecret-tools \
    libdbus-1-dev pkg-config protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .
RUN cd rust && cargo build --locked --release -p revault_bindings -p revault_tooling
RUN install -m 0755 rust/target/release/revault-tool /usr/local/bin/revault-tool \
    && revault-tool release package-native --repository . \
      --version 0.1.0 --target linux-x86_64-gnu \
      --library rust/target/release/librevault_api.so \
      --static-library rust/target/release/librevault_api.a --output /tmp/native \
    && revault-tool release install-native \
      --archive /tmp/native/revault-api-native-0.1.0-linux-x86_64-gnu.tar.gz \
      --prefix /usr/local \
    && ldconfig
RUN cmake -S bindings/e2e/c -B /tmp/revault-c-conformance \
    -DCMAKE_BUILD_TYPE=Release \
    && cmake --build /tmp/revault-c-conformance -j2

ENV LD_LIBRARY_PATH=/usr/local/lib
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/lib
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "c"]
