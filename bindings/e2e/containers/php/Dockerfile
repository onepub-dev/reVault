FROM rust:1.95-bookworm AS native
RUN apt-get update && apt-get install -y --no-install-recommends libdbus-1-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
RUN cd rust && cargo build --locked --release -p revault_bindings -p revault_tooling
RUN rust/target/release/revault-tool release package-native --repository . \
      --version 0.2.0 --target linux-x86_64-gnu \
      --library rust/target/release/librevault_api.so \
      --static-library rust/target/release/librevault_api.a --output /tmp/native

FROM composer:2 AS composer

FROM php:8.3-cli-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 git unzip libffi-dev \
    && docker-php-ext-configure ffi --with-ffi \
    && docker-php-ext-install ffi \
    && rm -rf /var/lib/apt/lists/*
COPY --from=composer /usr/bin/composer /usr/local/bin/composer
WORKDIR /workspace
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.2.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.2.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN rm -rf bindings/php \
    && cp -a /tmp/packages/composer bindings/php \
    && cd bindings/e2e/php \
    && composer install --no-interaction --no-progress --prefer-dist --optimize-autoloader
ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/workspace/bindings/e2e/php/vendor/onepub/revault-api/native/linux-x86_64-gnu
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "php"]
