package com.onepub.revault.e2e;

import com.onepub.revault.CacheMode;
import com.onepub.revault.Revault;
import com.onepub.revault.RevaultException;
import com.onepub.revault.WorkerPolicy;
import com.onepub.revault.WorkloadProfile;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.Arrays;

/** Verifies a Rust-produced plaintext lockbox through the public Java facade. */
public final class PlaintextFileConformance {
  private PlaintextFileConformance() { }

  /** Arguments: native library, existing lockbox, logical entry, expected host file. */
  public static void main(String[] args) throws Exception {
    if (args.length != 4) throw new IllegalArgumentException("Expected library, archive, entry, expected file");
    var api = Revault.load(Path.of(args[0]));
    var options = new Revault.LockboxOptions(CacheMode.DISABLED, 0,
        WorkloadProfile.INTERACTIVE, WorkerPolicy.SINGLE, 0);
    var archive = Path.of(args[1]);
    var before = digest(archive);
    try (var expected = Files.newInputStream(Path.of(args[3]));
        var box = api.openUnencryptedLockboxFile(archive.toString(), options)) {
      long offset = 0;
      while (true) {
        var wanted = expected.readNBytes(65536);
        if (wanted.length == 0) break;
        var actual = box.readRange(args[2], offset, wanted.length);
        if (!Arrays.equals(wanted, actual)) throw new AssertionError("Entry bytes differ at " + offset);
        offset += wanted.length;
      }
      if (box.readRange(args[2], offset, 1).length != 0) throw new AssertionError("Unexpected trailing bytes");
      try {
        box.addFile("/forbidden-write", new byte[] {1}, false);
        throw new AssertionError("Read-only lockbox accepted mutation");
      } catch (RevaultException expectedFailure) { /* Native read-only contract. */ }
    }
    if (!Arrays.equals(before, digest(archive))) throw new AssertionError("Reader modified archive");
    System.out.println("PASS plaintext file range reads and read-only ownership");
  }

  private static byte[] digest(Path path) throws Exception {
    var hash = MessageDigest.getInstance("SHA-256");
    try (var input = Files.newInputStream(path)) {
      var chunk = new byte[65536];
      int count;
      while ((count = input.read(chunk)) != -1) hash.update(chunk, 0, count);
    }
    return hash.digest();
  }
}
