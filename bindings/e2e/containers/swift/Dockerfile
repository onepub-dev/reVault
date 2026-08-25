FROM revault-e2e-native:local AS native

FROM swift:6.1-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY bindings/swift/Package.swift bindings/swift/Package.swift
RUN cd bindings/swift && swift package resolve
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.2.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.2.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages \
    && revault-tool release install-native --archive /tmp/native/revault-api-native-0.2.0-linux-x86_64-gnu.tar.gz --prefix /usr/local
RUN cd /tmp/packages/swift \
    && LIBRARY_PATH=/usr/local/lib swift build -c release \
    && cd /workspace \
    && revault-tool bindings check --repository .
ENV LD_LIBRARY_PATH=/usr/local/lib
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/lib
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "swift"]
