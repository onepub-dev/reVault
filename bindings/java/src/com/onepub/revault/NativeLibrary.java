package com.onepub.revault;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Locale;

/** Resolves an explicit native library or extracts the matching JAR resource. */
final class NativeLibrary {
  private NativeLibrary() {}

  static String resolve() {
    String override = System.getProperty("revault.library");
    if (override == null || override.isBlank()) override = System.getenv("REVAULT_LIBRARY");
    if (override != null && !override.isBlank()) return override;
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
                + "; set REVAULT_LIBRARY for development");
      }
      boolean keepExtracted = Boolean.getBoolean("revault.keepExtracted");
      Path directory;
      if (keepExtracted) {
        directory = Path.of(System.getProperty("java.io.tmpdir"), "revault-native");
        Files.createDirectories(directory);
      } else {
        directory = Files.createTempDirectory("revault-native-");
      }
      Path extracted = directory.resolve(library);
      Files.copy(source, extracted, StandardCopyOption.REPLACE_EXISTING);
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
