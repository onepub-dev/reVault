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

FROM swift:6.1-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 protobuf-compiler python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY bindings/swift/Package.swift bindings/swift/Package.swift
RUN cd bindings/swift && swift package resolve \
    && swift build --package-path .build/checkouts/swift-protobuf -c release --product protoc-gen-swift
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.1.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.1.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages \
    && revault-tool release install-native --archive /tmp/native/revault-api-native-0.1.0-linux-x86_64-gnu.tar.gz --prefix /usr/local
RUN cd /tmp/packages/swift \
    && protoc -I /workspace/bindings/proto \
      --plugin=protoc-gen-swift=/workspace/bindings/swift/.build/checkouts/swift-protobuf/.build/release/protoc-gen-swift \
      --swift_opt=Visibility=Public \
      --swift_out=Sources/RevaultAPI /workspace/bindings/proto/revault_bindings.proto \
    && cp Sources/RevaultAPI/revault_bindings.pb.swift \
      /workspace/bindings/swift/Sources/RevaultAPI/revault_bindings.pb.swift \
    && LIBRARY_PATH=/usr/local/lib swift build -c release \
    && cd /workspace \
    && revault-tool bindings check --repository .
ENV LD_LIBRARY_PATH=/usr/local/lib
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/lib
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "swift"]
