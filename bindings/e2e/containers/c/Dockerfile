FROM revault-e2e-native:local

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential cmake dbus-x11 gnome-keyring libsecret-tools \
    libdbus-1-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .
RUN revault-tool release install-native \
      --archive /tmp/native/revault-api-native-0.2.0-linux-x86_64-gnu.tar.gz \
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
