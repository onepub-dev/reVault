package com.onepub.revault;

import java.lang.foreign.Arena;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.invoke.MethodHandle;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/** Java 22+ FFM wrapper for the shared reVault C ABI. */
final class RevaultBindings {
  public enum ProfileGenerationStatus { ACTIVE, RETIRED, COMPROMISED }
  public record ProfileGeneration(int index, ProfileGenerationStatus status, byte[] contactFingerprint,
      long createdAtUnixMs, Long retiredAtUnixMs) {}
  public record ProfileHistory(String name, int activeGeneration,
      java.util.List<ProfileGeneration> generations) {}
  private static final java.lang.foreign.MemoryLayout BUFFER = java.lang.foreign.MemoryLayout.structLayout(
      java.lang.foreign.ValueLayout.ADDRESS.withName("ptr"),
      java.lang.foreign.ValueLayout.JAVA_LONG.withName("len"));
  private static final java.lang.foreign.MemoryLayout PATH = java.lang.foreign.ValueLayout.ADDRESS;
  private final Linker linker = Linker.nativeLinker();
  private final SymbolLookup symbols;
  private final MethodHandle create;
  private final MethodHandle free;
  private final MethodHandle addFile;
  private final MethodHandle commit;
  private final MethodHandle getFile;
  private final MethodHandle bufferFree;
  private final MethodHandle openVaultDirectory;
  private final MethodHandle listProfileGenerations;
  private final MethodHandle freeVaultDirectory;

  public RevaultBindings(String library) {
    symbols = SymbolLookup.libraryLookup(library, Arena.global());
    create = linker.downcallHandle(symbols.find("lockbox_create").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.ADDRESS,
            java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));
    free = linker.downcallHandle(symbols.find("lockbox_free").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.ADDRESS));
    addFile = linker.downcallHandle(symbols.find("lockbox_add_file").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_BOOLEAN,
            java.lang.foreign.ValueLayout.ADDRESS, PATH, java.lang.foreign.ValueLayout.JAVA_LONG,
            java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG,
            java.lang.foreign.ValueLayout.JAVA_BOOLEAN));
    commit = linker.downcallHandle(symbols.find("lockbox_commit").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_BOOLEAN,
            java.lang.foreign.ValueLayout.ADDRESS));
    getFile = linker.downcallHandle(symbols.find("lockbox_get_file").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.of(BUFFER, java.lang.foreign.ValueLayout.ADDRESS,
            PATH, java.lang.foreign.ValueLayout.JAVA_LONG));
    bufferFree = linker.downcallHandle(symbols.find("buffer_free").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.ofVoid(BUFFER));
    openVaultDirectory = linker.downcallHandle(symbols.find("vault_directory_open_or_create").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.ADDRESS,
            java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG,
            java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));
    listProfileGenerations = linker.downcallHandle(symbols.find("vault_directory_list_profile_generations").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.of(BUFFER, java.lang.foreign.ValueLayout.ADDRESS,
            PATH, java.lang.foreign.ValueLayout.JAVA_LONG));
    freeVaultDirectory = linker.downcallHandle(symbols.find("vault_directory_free").orElseThrow(),
        java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.ADDRESS));
  }

  /** Returns any exported ABI symbol for advanced/generated callers. */
  public MemorySegment symbol(String name) { return symbols.find(name).orElseThrow(); }
  public SymbolLookup symbolLookup() { return symbols; }

  public Lockbox create(byte[] key) throws Throwable {
    try (var arena = Arena.ofConfined()) {
      var bytes = arena.allocate(key.length);
      bytes.copyFrom(MemorySegment.ofArray(key));
      return new Lockbox((MemorySegment) create.invoke(bytes, (long) key.length), free);
    }
  }

  public void addFile(Lockbox lockbox, String path, byte[] data, boolean replace) throws Throwable {
    lockbox.addFile(path, data, replace, addFile);
  }

  public void commit(Lockbox lockbox) throws Throwable {
    lockbox.commit(commit);
  }

  public byte[] getFile(Lockbox lockbox, String path) throws Throwable {
    return lockbox.getFile(path, getFile, bufferFree);
  }

  public VaultDirectory openVaultDirectory(String root, byte[] password) throws Throwable {
    try (var arena = Arena.ofConfined()) {
      var rootBytes = arena.allocateFrom(root);
      var secret = arena.allocate(password.length);
      secret.copyFrom(MemorySegment.ofArray(password));
      var handle = (MemorySegment) openVaultDirectory.invoke(rootBytes,
          (long) root.getBytes(StandardCharsets.UTF_8).length, secret, (long) password.length);
      if (handle.address() == 0) throw new IllegalStateException("reVault vault directory open failed");
      return new VaultDirectory(handle, listProfileGenerations, freeVaultDirectory, bufferFree);
    }
  }

  public static final class VaultDirectory implements AutoCloseable {
    private MemorySegment handle;
    private final MethodHandle list;
    private final MethodHandle free;
    private final MethodHandle bufferFree;
    VaultDirectory(MemorySegment handle, MethodHandle list, MethodHandle free, MethodHandle bufferFree) {
      this.handle = handle; this.list = list; this.free = free; this.bufferFree = bufferFree;
    }
    public ProfileHistory listProfileGenerations(String name) throws Throwable {
      try (var arena = Arena.ofConfined()) {
        var bytes = arena.allocateFrom(name);
        var result = (MemorySegment) list.invoke(handle, bytes, (long) name.getBytes(StandardCharsets.UTF_8).length);
        var pointer = result.get(java.lang.foreign.ValueLayout.ADDRESS, 0).address();
        var length = result.get(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS.byteSize());
        if (pointer == 0) throw new IllegalStateException("reVault profile history read failed");
        var value = decodeProfileHistory(MemorySegment.ofAddress(pointer).reinterpret(length).toArray(java.lang.foreign.ValueLayout.JAVA_BYTE));
        bufferFree.invoke(result);
        return value;
      }
    }
    @Override public void close() { if (handle != null) { try { free.invoke(handle); } catch (Throwable error) { throw new RuntimeException(error); } handle = null; } }
  }

  private static ProfileHistory decodeProfileHistory(byte[] frame) {
    if (frame.length < 12 || frame[0] != 'L' || frame[1] != 'B' || frame[2] != 'W' || frame[3] != 'F') throw new IllegalArgumentException("invalid binding frame");
    var payloadLength = ((frame[8] & 255) << 24) | ((frame[9] & 255) << 16) | ((frame[10] & 255) << 8) | (frame[11] & 255);
    if (payloadLength + 12 != frame.length) throw new IllegalArgumentException("invalid binding frame length");
    var reader = new ProtoReader(frame, 12, frame.length);
    String name = ""; int active = 0; var generations = new ArrayList<ProfileGeneration>();
    while (reader.hasNext()) {
      long tag = reader.varint(); int field = (int)(tag >>> 3);
      if ((tag & 7) == 2) {
        byte[] bytes = reader.bytes();
        if (field == 1) name = new String(bytes, StandardCharsets.UTF_8);
        else if (field == 3) generations.add(decodeProfileGeneration(bytes));
      } else if ((tag & 7) == 0 && field == 2) active = (int)reader.varint();
      else reader.skip(tag & 7);
    }
    return new ProfileHistory(name, active, List.copyOf(generations));
  }

  private static ProfileGeneration decodeProfileGeneration(byte[] bytes) {
    var reader = new ProtoReader(bytes, 0, bytes.length); int index = 0; var status = ProfileGenerationStatus.ACTIVE; byte[] fingerprint = new byte[0]; long created = 0; Long retired = null;
    while (reader.hasNext()) { long tag = reader.varint(); int field = (int)(tag >>> 3); if ((tag & 7) == 2) { byte[] value = reader.bytes(); if (field == 2) status = ProfileGenerationStatus.valueOf(new String(value, StandardCharsets.UTF_8).toUpperCase()); else if (field == 3) fingerprint = value; } else if ((tag & 7) == 0) { long value = reader.varint(); if (field == 1) index = (int)value; else if (field == 4) created = value; else if (field == 5) retired = value; } }
    return new ProfileGeneration(index, status, fingerprint, created, retired);
  }

  private static final class ProtoReader {
    private final byte[] bytes; private int offset; private final int end;
    ProtoReader(byte[] bytes, int offset, int end) { this.bytes = bytes; this.offset = offset; this.end = end; }
    boolean hasNext() { return offset < end; }
    long varint() { long value = 0; int shift = 0; while (offset < end) { int b = bytes[offset++] & 255; value |= (long)(b & 127) << shift; if ((b & 128) == 0) return value; shift += 7; } throw new IllegalArgumentException("truncated protobuf"); }
    byte[] bytes() { int length = (int)varint(); var value = java.util.Arrays.copyOfRange(bytes, offset, offset + length); offset += length; return value; }
    void skip(long wire) { if (wire == 0) varint(); else if (wire == 2) { int length = (int)varint(); offset += length; } else throw new IllegalArgumentException("unsupported protobuf wire type"); }
  }

  public static final class Lockbox implements AutoCloseable {
    private MemorySegment handle;
    private final MethodHandle free;
    Lockbox(MemorySegment handle, MethodHandle free) { this.handle = handle; this.free = free; }

    public void addFile(String path, byte[] data, boolean replace, MethodHandle addFile) throws Throwable {
      try (var arena = Arena.ofConfined()) {
        var pathBytes = arena.allocateFrom(path);
        var content = arena.allocate(data.length);
        content.copyFrom(MemorySegment.ofArray(data));
        boolean ok = (boolean) addFile.invoke(handle, pathBytes, (long) path.getBytes(java.nio.charset.StandardCharsets.UTF_8).length, content, (long) data.length, replace);
        if (!ok) throw new IllegalStateException("reVault lockbox operation failed");
      }
    }

    public void commit(MethodHandle commit) throws Throwable {
      if (!(boolean) commit.invoke(handle)) throw new IllegalStateException("reVault commit failed");
    }

    public byte[] getFile(String path, MethodHandle getFile, MethodHandle bufferFree) throws Throwable {
      try (var arena = Arena.ofConfined()) {
        var pathBytes = arena.allocateFrom(path);
        var result = (MemorySegment) getFile.invoke(handle, pathBytes, (long) path.getBytes(java.nio.charset.StandardCharsets.UTF_8).length);
        var pointer = result.get(java.lang.foreign.ValueLayout.ADDRESS, 0).address();
        var length = result.get(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS.byteSize());
        if (pointer == 0) throw new IllegalStateException("reVault read failed");
        var bytes = MemorySegment.ofAddress(pointer).reinterpret(length).toArray(java.lang.foreign.ValueLayout.JAVA_BYTE);
        bufferFree.invoke(result);
        return bytes;
      }
    }
    @Override public void close() {
      if (handle != null) {
        try { free.invoke(handle); } catch (Throwable error) { throw new RuntimeException(error); }
        handle = null;
      }
    }
  }
}
