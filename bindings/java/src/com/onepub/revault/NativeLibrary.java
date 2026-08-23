package com.onepub.revault;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;

/** Resolves the carrier shipped in the installed JAR. */
final class NativeLibrary {
  private NativeLibrary() {}

  static String resolve() {
    String os = System.getProperty("os.name").toLowerCase(Locale.ROOT);
    String arch = System.getProperty("os.arch").toLowerCase(Locale.ROOT);
    String cpu = switch (arch) {
      case "amd64", "x86_64" -> "x86_64";
      case "aarch64", "arm64" -> "aarch64";
      default -> throw new IllegalStateException("unsupported reVault architecture: " + arch);
    };
    String target;
    String library;
    if (os.contains("win")) {
      target = "windows-" + cpu + "-msvc";
      library = "revault_api.dll";
    } else if (os.contains("mac") || os.contains("darwin")) {
      target = "macos-" + cpu;
      library = "librevault_api.dylib";
    } else if (os.contains("linux")) {
      target = "linux-" + cpu + "-gnu";
      library = "librevault_api.so";
    } else {
      throw new IllegalStateException("unsupported reVault operating system: " + os);
    }
    String resource = "/META-INF/native/" + target + "/" + library;
    try (InputStream source = NativeLibrary.class.getResourceAsStream(resource)) {
      if (source == null) {
        throw new IllegalStateException(
            "revault-api native carrier is missing for " + target
                + "; install the matching revault-api carrier for this target");
      }
      // The conformance runner sets this flag so it can record the exact
      // carrier that the installed JAR loaded. Reuse an existing file rather
      // than replacing it: each package suite starts several JVMs, and a
      // replacement here would make the evidence depend on test setup rather
      // than the installed package. Normal applications continue to receive
      // an isolated, delete-on-exit extraction directory.
      boolean keepExtracted = Boolean.getBoolean("revault.keepExtracted");
      Path directory = keepExtracted
          ? Path.of(System.getProperty("java.io.tmpdir"), "revault-native")
          : Files.createTempDirectory("revault-native-");
      Files.createDirectories(directory);
      Path extracted = directory.resolve(library);
      if (!Files.exists(extracted)) Files.copy(source, extracted);
      if (!keepExtracted) {
        extracted.toFile().deleteOnExit();
        directory.toFile().deleteOnExit();
      }
      return extracted.toAbsolutePath().toString();
    } catch (IOException error) {
      throw new IllegalStateException("could not extract " + resource, error);
    }
  }
}
