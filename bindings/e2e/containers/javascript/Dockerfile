FROM rust:1.95-bookworm AS native
RUN apt-get update && apt-get install -y --no-install-recommends libdbus-1-dev pkg-config protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY rust rust
COPY bindings/proto bindings/proto
COPY bindings/release bindings/release
COPY bindings/ruby/native/revault_ruby_shim.c bindings/ruby/native/revault_ruby_shim.c
RUN cd rust && cargo build --locked --release -p revault_bindings -p revault_tooling
RUN rust/target/release/revault-tool release package-native --repository . \
      --version 0.1.0 --target linux-x86_64-gnu \
      --library rust/target/release/librevault_api.so \
      --static-library rust/target/release/librevault_api.a --output /tmp/native

FROM node:22-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.1.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.1.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN mkdir -p /opt/revault-js-consumer \
    && npm pack /tmp/packages/npm/revault-api --pack-destination /tmp \
    && npm pack /tmp/packages/npm/revault-api-native-linux-x86_64-gnu --pack-destination /tmp \
    && cd /opt/revault-js-consumer \
    && npm init -y \
    && npm install --omit=optional /tmp/onepub-dev-revault-api-0.1.0.tgz \
      /tmp/onepub-dev-revault-api-native-linux-x86_64-gnu-0.1.0.tgz
ENV REVAULT_E2E_MODULE=/opt/revault-js-consumer/node_modules/@onepub-dev/revault-api/index.js
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/opt/revault-js-consumer/node_modules/@onepub-dev/revault-api-native-linux-x86_64-gnu/lib
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "javascript"]
