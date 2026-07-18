FROM rust:1.95-bookworm AS native
RUN apt-get update && apt-get install -y --no-install-recommends libdbus-1-dev pkg-config protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
RUN cd rust && cargo build --locked --release -p revault_bindings -p revault_tooling
RUN rust/target/release/revault-tool release package-native --repository . \
      --version 0.1.0 --target linux-x86_64-gnu \
      --library rust/target/release/librevault_api.so \
      --static-library rust/target/release/librevault_api.a --output /tmp/native

FROM ruby:3.3-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 \
    && gem install google-protobuf -v '~> 3.25' --no-document \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.1.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.1.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN cd /tmp/packages/ruby/linux-x86_64-gnu \
    && gem build revault_api.gemspec \
    && gem install ./revault_api-0.1.0-*.gem --no-document
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/usr/local/bundle/gems/revault_api-0.1.0-x86_64-linux/native/linux-x86_64-gnu
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "ruby"]
