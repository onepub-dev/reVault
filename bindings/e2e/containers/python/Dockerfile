FROM revault-e2e-native:local

RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-dev pkg-config \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
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
