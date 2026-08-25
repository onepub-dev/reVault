FROM revault-e2e-native:local AS native

FROM ruby:3.3-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.2.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.2.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN cd /tmp/packages/ruby/linux-x86_64-gnu \
    && gem build revault_api.gemspec \
    && gem install ./revault_api-0.2.0-*.gem --no-document
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/bundle/gems/revault_api-0.2.0-x86_64-linux/native/linux-x86_64-gnu
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "ruby"]
