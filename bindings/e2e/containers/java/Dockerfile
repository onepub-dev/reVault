FROM rust:1.95-bookworm AS native

RUN apt-get update && apt-get install -y --no-install-recommends \
    libdbus-1-dev pkg-config protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
RUN cd rust && cargo build --locked --release -p revault_bindings -p revault_tooling
RUN rust/target/release/revault-tool release package-native --repository . \
      --version 0.1.0 --target linux-x86_64-gnu \
      --library rust/target/release/librevault_api.so \
      --static-library rust/target/release/librevault_api.a --output /tmp/native

FROM maven:3.9-eclipse-temurin-22
RUN apt-get update && apt-get install -y --no-install-recommends \
    dbus-x11 gnome-keyring libsecret-tools libdbus-1-3 python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY . .
COPY --from=native /tmp/native /tmp/native
COPY --from=native /workspace/rust/target/release/revault-tool /usr/local/bin/revault-tool
RUN revault-tool release stage-ecosystems --allow-partial --repository . \
      --version 0.1.0 --output /tmp/ecosystem /tmp/native/*.tar.gz \
    && revault-tool release assemble-packages --allow-partial --version 0.1.0 \
      --layout /tmp/ecosystem --source . --output /tmp/packages
RUN mvn -q dependency:get -Dartifact=com.google.protobuf:protobuf-java:3.21.12 \
    && mkdir -p /tmp/revault-api-classes /opt/revault-java \
    && javac --release 22 \
      -cp /root/.m2/repository/com/google/protobuf/protobuf-java/3.21.12/protobuf-java-3.21.12.jar \
      -d /tmp/revault-api-classes \
      $(find /tmp/packages/maven/java/src /tmp/packages/maven/java/generated -name '*.java' -print) \
    && jar --create --file /tmp/revault-api-0.1.0.jar \
      -C /tmp/revault-api-classes . -C /tmp/packages/maven/java/resources . \
    && mvn -q install:install-file -Dfile=/tmp/revault-api-0.1.0.jar \
      -DgroupId=dev.onepub -DartifactId=revault-api -Dversion=0.1.0 -Dpackaging=jar \
    && javac --release 22 \
      -cp /root/.m2/repository/dev/onepub/revault-api/0.1.0/revault-api-0.1.0.jar:/root/.m2/repository/com/google/protobuf/protobuf-java/3.21.12/protobuf-java-3.21.12.jar \
      -d /opt/revault-java bindings/e2e/java/Conformance.java

ENV XDG_RUNTIME_DIR=/tmp/revault-runtime
ENV REVAULT_E2E_NATIVE_ROOT=/tmp/revault-java-extract
ENV REVAULT_E2E_NATIVE_FILE=librevault_api.so
ENV REVAULT_E2E_NATIVE_KIND=dynamic
CMD ["revault-tool", "e2e", "container", "--language", "java"]
