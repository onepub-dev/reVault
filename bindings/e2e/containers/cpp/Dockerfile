FROM revault-e2e-native:local

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential cmake dbus-x11 gnome-keyring libsecret-tools \
    libdbus-1-dev pkg-config python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .
RUN revault-tool release install-native \
      --archive /tmp/native/revault-api-native-0.2.0-linux-x86_64-gnu.tar.gz \
      --prefix /usr/local \
    && revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.2.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.2.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages \
    && cmake -S /tmp/packages/native-sdk/cpp -B /tmp/revault-api-cpp -DCMAKE_BUILD_TYPE=Release \
    && cmake --build /tmp/revault-api-cpp -j2 \
    && cmake --install /tmp/revault-api-cpp --prefix /usr/local \
    && ldconfig
RUN cmake -S bindings/e2e/cpp -B /tmp/revault-cpp-conformance \
    -DCMAKE_BUILD_TYPE=Release \
    && cmake --build /tmp/revault-cpp-conformance -j2

ENV LD_LIBRARY_PATH=/usr/local/lib
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/lib
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "cpp"]
