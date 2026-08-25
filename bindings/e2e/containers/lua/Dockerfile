FROM revault-e2e-native:local AS native

FROM debian:bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 \
    luajit luarocks python3 build-essential liblua5.1-0-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.2.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.2.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN cd /tmp/packages/lua/linux-x86_64-gnu \
    && luarocks --lua-version=5.1 make revault_api-0.2.0-1.rockspec \
    && cd /workspace \
    && revault-tool bindings check --repository .
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/lib/lua/5.1
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "lua"]
