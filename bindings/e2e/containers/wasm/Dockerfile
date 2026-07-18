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

FROM rust:1.95-bookworm AS wasm
WORKDIR /workspace
COPY rust rust
COPY bindings/e2e/operations.tsv bindings/e2e/operations.tsv
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.121 --locked \
    && cd rust \
    && cargo build --locked -p revault_wasm_bindings --target wasm32-unknown-unknown --release \
    && wasm-bindgen target/wasm32-unknown-unknown/release/revault_wasm_bindings.wasm \
      --target nodejs --out-dir /tmp/revault-wasm \
    && mv /tmp/revault-wasm/revault_wasm_bindings.js /tmp/revault-wasm/revault_wasm_bindings.cjs

FROM node:22-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
COPY --from=wasm /tmp/revault-wasm bindings/wasm/generated
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.1.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.1.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN revault-tool bindings check --repository . \
    && mkdir -p /opt/revault-wasm-consumer \
    && npm pack /tmp/packages/npm/revault-api --pack-destination /tmp \
    && npm pack /tmp/packages/npm/revault-api-native-linux-x86_64-gnu --pack-destination /tmp \
    && npm pack /tmp/packages/npm/revault-api-wasm --pack-destination /tmp \
    && cd /opt/revault-wasm-consumer \
    && npm init -y \
    && npm install --omit=optional /tmp/onepub-dev-revault-api-0.1.0.tgz \
      /tmp/onepub-dev-revault-api-native-linux-x86_64-gnu-0.1.0.tgz \
    && npm install --omit=optional /tmp/onepub-dev-revault-api-wasm-0.1.0.tgz
ENV REVAULT_E2E_MODULE=/opt/revault-wasm-consumer/node_modules/@onepub-dev/revault-api-wasm/index.js
ENV REVAULT_E2E_LANGUAGE=wasm
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/opt/revault-wasm-consumer/node_modules/@onepub-dev/revault-api-wasm/generated
ENV REVAULT_E2E_NATIVE_FILE=revault_wasm_bindings_bg.wasm
ENV REVAULT_E2E_NATIVE_KIND=wasm
CMD ["revault-tool", "e2e", "container", "--language", "wasm"]
