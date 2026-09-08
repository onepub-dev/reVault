//! Regenerate the native file-operation additions to the existing binding surfaces.
//! Run from the repository root: rustc --edition 2021 bindings/api/generate_file_operation.rs -o /tmp/revault-file-codegen && /tmp/revault-file-codegen
//! Afterwards run Dart's formatter (language version 3.10) on revault_native.dart
//! and binding_operations.dart, gofmt on revault.go/revault_native.go, and
//! dotnet format on bindings/csharp/RevaultBindings.csproj.
use std::{fs, path::Path};
fn insert(path: &str, anchor: &str, id: &str, comment: &str, body: &str) {
    let mut s = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let start = format!("{comment} BEGIN generated file operation {id}\n");
    let end = format!("{comment} END generated file operation {id}\n");
    let block = format!("{start}{body}\n{end}");
    if let Some(a) = s.find(&start) {
        let b = a + s[a..].find(&end).unwrap() + end.len();
        s.replace_range(a..b, &block);
    } else {
        let i = s
            .find(anchor)
            .unwrap_or_else(|| panic!("anchor {anchor:?} missing in {path}"));
        s.insert_str(i, &block);
    }
    fs::write(path, s).unwrap();
}
const ARGS:&str="const char *path, size_t path_len, const char *mode, size_t mode_len, const char *credential, size_t credential_len, const uint8_t *secret, size_t secret_len, const void *contact, const void *signer, const char *cache_mode, size_t cache_len, uint64_t cache_bytes, const char *workload, size_t workload_len, const char *worker, size_t worker_len, size_t jobs";
const TYPES:&str="void *, size_t, void *, size_t, void *, size_t, void *, size_t, void *, void *, void *, size_t, uint64_t, void *, size_t, void *, size_t, size_t";
fn main() {
    // PHP FFI represents a failed pointer-returning call as null, not CData.
    let php = "bindings/php/src/BindingOperations.php";
    let source = fs::read_to_string(php)
        .unwrap()
        .replace(
            "private function requireHandle(CData $value)",
            "private function requireHandle(?CData $value)",
        )
        .replace(
            "if (FFI::isNull($value))",
            "if ($value === null || FFI::isNull($value))",
        );
    fs::write(php, source).unwrap();

    let declaration=format!("/** Native file operation; mode is open/create/replace. A signer selects exclusive write access. See file_api.rs for credential, tuning and ownership contracts. */\nvoid *lockbox_file({ARGS});");
    for p in [
        "rust/revault_bindings/revault_api.h",
        "bindings/php/revault_ffi.h",
    ] {
        insert(p, "void *lockbox_open(", "declaration", "//", &declaration);
    }
    for p in [
        "rust/revault_bindings/revault_api.h",
        "bindings/php/revault_ffi.h",
    ] {
        let s=fs::read_to_string(p).unwrap().replace("// END generated file operation declaration\nvoid *lockbox_open", "// END generated file operation declaration\n/** Opens existing in-memory archive bytes. */\nvoid *lockbox_open");
        fs::write(p, s).unwrap();
    }
    for inventory in [
        "rust/revault_wasm_bindings/operations.tsv",
        "bindings/e2e/operations.tsv",
    ] {
        let mut s = fs::read_to_string(inventory).unwrap();
        if !s.contains("lockbox_file\t") {
            s.push_str(&format!(
                "lockbox_file\tarchive.lifecycle\tvoid *\t{ARGS}\n"
            ));
            fs::write(inventory, s).unwrap();
        }
    }
    insert(
        "bindings/javascript/native.js",
        "let lockbox_open;",
        "symbol",
        "//",
        "let lockbox_file;",
    );
    insert(
        "bindings/javascript/native.js",
        "  lockbox_open =",
        "load",
        "//",
        &format!("  lockbox_file = library.func('void * lockbox_file({TYPES})');"),
    );
    insert(
        "bindings/javascript/native.js",
        "  lockboxOpen(",
        "route",
        "//",
        r#"  lockboxFile(path, mode, credential, secret, contact, signer, cacheMode, cacheBytes, workload, worker, jobs) {
    return requireHandle(lockbox_file(Buffer.from(path), Buffer.byteLength(path), Buffer.from(mode), Buffer.byteLength(mode), Buffer.from(credential), Buffer.byteLength(credential), Buffer.from(secret), Buffer.byteLength(secret), contact, signer, Buffer.from(cacheMode), Buffer.byteLength(cacheMode), cacheBytes, Buffer.from(workload), Buffer.byteLength(workload), Buffer.from(worker), Buffer.byteLength(worker), jobs));
  }"#,
    );
    insert(
        "bindings/python/revault_api/_revault_native.py",
        "    library.lockbox_open.argtypes",
        "abi",
        "#",
        r#"    library.lockbox_file.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t]
    library.lockbox_file.restype = ctypes.c_void_p"#,
    );
    insert("bindings/python/revault_api/facade.py","    'lockbox_open':", "route","#", "    'lockbox_file': (('text', 'text', 'text', 'bytes', 'value', 'value', 'text', 'value', 'text', 'text', 'value'), 'handle:Lockbox', False),");
    insert(
        "bindings/ruby/lib/revault/binding_operations.rb",
        "    extern 'void * lockbox_open(",
        "abi",
        "#",
        &format!("    extern 'void * lockbox_file({TYPES})'"),
    );
    insert(
        "bindings/ruby/lib/revault/binding_operations.rb",
        "    def lockbox_open(",
        "route",
        "#",
        r#"    def lockbox_file(path, mode, credential, secret, contact, signer, cache_mode, cache_bytes, workload, worker, jobs)
      require_handle(Native.lockbox_file(Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[mode], mode.bytesize, Fiddle::Pointer[credential], credential.bytesize, Fiddle::Pointer[secret], secret.bytesize, contact || 0, signer || 0, Fiddle::Pointer[cache_mode], cache_mode.bytesize, cache_bytes, Fiddle::Pointer[workload], workload.bytesize, Fiddle::Pointer[worker], worker.bytesize, jobs))
    end"#,
    );
    insert(
        "bindings/php/src/BindingOperations.php",
        "    public function lockboxOpen(",
        "route",
        "//",
        r#"    public function lockboxFile(string $path, string $mode, string $credential, string $secret, ?CData $contact, ?CData $signer, string $cacheMode, int $cacheBytes, string $workload, string $worker, int $jobs): CData
    {
        return $this->withBytes($secret, fn(CData $pointer, int $length) => $this->requireHandle($this->ffi->lockbox_file($path, strlen($path), $mode, strlen($mode), $credential, strlen($credential), $pointer, $length, $contact, $signer, $cacheMode, strlen($cacheMode), $cacheBytes, $workload, strlen($workload), $worker, strlen($worker), $jobs)));
    }"#,
    );
    insert(
        "bindings/lua/revault_api.lua",
        "void * lockbox_open(",
        "abi",
        "//",
        &declaration,
    );
    insert(
        "bindings/lua/revault_api.lua",
        "function Operations:lockbox_open(",
        "route",
        "--",
        r#"function Operations:lockbox_file(path, mode, credential, secret, contact, signer, cache_mode, cache_bytes, workload, worker, jobs)
  local value = native.lockbox_file(path, #path, mode, #mode, credential, #credential, secret, #secret, contact, signer, cache_mode, #cache_mode, cache_bytes, workload, #workload, worker, #worker, jobs)
  if value == nil then error(last_error(), 2) end
  return value
end"#,
    );
    dart();
    remaining();
    assert!(Path::new("bindings/AGENTS.md").exists());
}
fn dart() {
    let native = "bindings/dart/lib/src/revault_native.dart";
    let nt="ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Uint64, ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Size)";
    let dt = nt.replace("ffi.Size", "int").replace("ffi.Uint64", "int");
    let params="ffi.Pointer<ffi.Uint8> path, int pathLen, ffi.Pointer<ffi.Uint8> mode, int modeLen, ffi.Pointer<ffi.Uint8> credential, int credentialLen, ffi.Pointer<ffi.Uint8> secret, int secretLen, ffi.Pointer<ffi.Void> contact, ffi.Pointer<ffi.Void> signer, ffi.Pointer<ffi.Uint8> cacheMode, int cacheLen, int cacheBytes, ffi.Pointer<ffi.Uint8> workload, int workloadLen, ffi.Pointer<ffi.Uint8> worker, int workerLen, int jobs";
    insert(native,"typedef _ApiAbiVersionNative", "types","//",&format!("typedef _LockboxFileNative = {nt};\ntypedef _LockboxFileDart = {dt};\n@ffi.Native<_LockboxFileNative>(symbol: 'lockbox_file', assetId: 'package:revault_api/src/revault_native.dart')\nexternal ffi.Pointer<ffi.Void> _native_lockbox_file({params});"));
    insert(native,"      _lockbox_open = library", "load","//","      _lockbox_file = library.lookupFunction<_LockboxFileNative, _LockboxFileDart>('lockbox_file'),");
    insert(
        native,
        "      _lockbox_open = _native_lockbox_open,",
        "asset",
        "//",
        "      _lockbox_file = _native_lockbox_file,",
    );
    insert(native,"  final _LockboxOpenDart", "field","//",&format!("  final _LockboxFileDart _lockbox_file;\n  ffi.Pointer<ffi.Void> lockbox_file({params}) => _lockbox_file(path, pathLen, mode, modeLen, credential, credentialLen, secret, secretLen, contact, signer, cacheMode, cacheLen, cacheBytes, workload, workloadLen, worker, workerLen, jobs);"));
    let mut body=String::from("  ffi.Pointer<ffi.Void> lockboxFile(String path, String mode, String credential, Uint8List secret, ffi.Pointer<ffi.Void> contact, ffi.Pointer<ffi.Void> signer, String cacheMode, int cacheBytes, String workload, String worker, int jobs) => ");
    for (n, k) in [
        ("path", "Text"),
        ("mode", "Text"),
        ("credential", "Text"),
        ("secret", "Bytes"),
        ("cacheMode", "Text"),
        ("workload", "Text"),
        ("worker", "Text"),
    ] {
        body += &format!("_with{k}({n}, ({n}Pointer, {n}Length) => ");
    }
    body+="_requireHandle(native.lockbox_file(pathPointer, pathLength, modePointer, modeLength, credentialPointer, credentialLength, secretPointer, secretLength, contact, signer, cacheModePointer, cacheModeLength, cacheBytes, workloadPointer, workloadLength, workerPointer, workerLength, jobs))";
    body += &")".repeat(7);
    body += ";";
    insert(
        "bindings/dart/lib/src/binding_operations.dart",
        "  ffi.Pointer<ffi.Void> lockboxOpen(",
        "route",
        "//",
        &body,
    );
}
fn remaining() {
    let cs = "bindings/csharp/RevaultNative.cs";
    insert(
        cs,
        "    public static extern IntPtr lockbox_open(",
        "abi",
        "//",
        r#"    public static extern IntPtr lockbox_file(IntPtr path, nuint pathLen, IntPtr mode, nuint modeLen, IntPtr credential, nuint credentialLen, IntPtr secret, nuint secretLen, IntPtr contact, IntPtr signer, IntPtr cacheMode, nuint cacheLen, ulong cacheBytes, IntPtr workload, nuint workloadLen, IntPtr worker, nuint workerLen, nuint jobs);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]"#,
    );
    let strings = [
        "path",
        "mode",
        "credential",
        "cacheMode",
        "workload",
        "worker",
    ];
    let mut body=String::from("    public unsafe IntPtr LockboxFile(string path, string mode, string credential, byte[] secret, IntPtr contact, IntPtr signer, string cacheMode, ulong cacheBytes, string workload, string worker, nuint jobs) {\n");
    for n in strings {
        body += &format!("        var {n}Bytes = Encoding.UTF8.GetBytes({n});\n");
    }
    for n in strings {
        body += &format!("        fixed (byte* {n}Pointer = {n}Bytes)\n");
    }
    body+="        fixed (byte* secretPointer = secret)\n        { return Require(RevaultNative.lockbox_file((IntPtr)pathPointer, (nuint)pathBytes.Length, (IntPtr)modePointer, (nuint)modeBytes.Length, (IntPtr)credentialPointer, (nuint)credentialBytes.Length, (IntPtr)secretPointer, (nuint)secret.Length, contact, signer, (IntPtr)cacheModePointer, (nuint)cacheModeBytes.Length, cacheBytes, (IntPtr)workloadPointer, (nuint)workloadBytes.Length, (IntPtr)workerPointer, (nuint)workerBytes.Length, jobs)); }\n    }";
    insert(
        "bindings/csharp/BindingOperations.cs",
        "    public unsafe IntPtr LockboxOpen(",
        "route",
        "//",
        &body,
    );
    insert(
        "bindings/java/src/com/onepub/revault/RevaultAbiSymbols.java",
        "    \"lockbox_open\",",
        "symbol",
        "//",
        "    \"lockbox_file\",",
    );
    let j = "bindings/java/src/com/onepub/revault/RevaultNativeApi.java";
    insert(
        j,
        "  public final MethodHandle lockbox_open;",
        "field",
        "//",
        "  public final MethodHandle lockbox_file;",
    );
    insert(
        j,
        "    lockbox_open =",
        "load",
        "//",
        r#"    lockbox_file = linker.downcallHandle(symbols.find("lockbox_file").orElseThrow(), FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG));"#,
    );
    insert(
        "bindings/java/src/com/onepub/revault/BindingOperations.java",
        "  public MemorySegment lockboxOpen(",
        "route",
        "//",
        r#"  public MemorySegment lockboxFile(String path, String mode, String credential, byte[] secret, MemorySegment contact, MemorySegment signer, String cacheMode, long cacheBytes, String workload, String worker, long jobs) {
    try (var arena = Arena.ofConfined()) {
      return require((MemorySegment) call(api.lockbox_file, text(arena, path), (long) path.getBytes(StandardCharsets.UTF_8).length, text(arena, mode), (long) mode.getBytes(StandardCharsets.UTF_8).length, text(arena, credential), (long) credential.getBytes(StandardCharsets.UTF_8).length, bytes(arena, secret), (long) secret.length, contact, signer, text(arena, cacheMode), (long) cacheMode.getBytes(StandardCharsets.UTF_8).length, cacheBytes, text(arena, workload), (long) workload.getBytes(StandardCharsets.UTF_8).length, text(arena, worker), (long) worker.getBytes(StandardCharsets.UTF_8).length, jobs));
    }
  }"#,
    );
    insert(
        "bindings/go/revault_native.go",
        "func (native) LockboxOpen(",
        "abi",
        "//",
        r#"func (native) LockboxFile(path unsafe.Pointer, pathLen C.size_t, mode unsafe.Pointer, modeLen C.size_t, credential unsafe.Pointer, credentialLen C.size_t, secret unsafe.Pointer, secretLen C.size_t, contact unsafe.Pointer, signer unsafe.Pointer, cacheMode unsafe.Pointer, cacheLen C.size_t, cacheBytes C.uint64_t, workload unsafe.Pointer, workloadLen C.size_t, worker unsafe.Pointer, workerLen C.size_t, jobs C.size_t) unsafe.Pointer {
 return C.lockbox_file((*C.char)(path), pathLen, (*C.char)(mode), modeLen, (*C.char)(credential), credentialLen, (*C.uint8_t)(secret), secretLen, contact, signer, (*C.char)(cacheMode), cacheLen, cacheBytes, (*C.char)(workload), workloadLen, (*C.char)(worker), workerLen, jobs)
}"#,
    );
    insert(
        "bindings/go/revault.go",
        "func Open(archive, key",
        "route",
        "//",
        r#"func lockboxFile(path, mode, credential string, secret []byte, contact, signer unsafe.Pointer, options LockboxOptions) (*Lockbox, error) {
 if err := options.Validate(); err != nil { return nil, err }
 return adoptLockbox(C.lockbox_file(charPointer(path), C.size_t(len(path)), charPointer(mode), C.size_t(len(mode)), charPointer(credential), C.size_t(len(credential)), bytePointer(secret), C.size_t(len(secret)), contact, signer, charPointer(options.CacheMode), C.size_t(len(options.CacheMode)), C.uint64_t(options.CacheBytes), charPointer(options.Workload), C.size_t(len(options.Workload)), charPointer(options.Worker), C.size_t(len(options.Worker)), C.size_t(options.Jobs)))
}

// Open opens serialized in-memory bytes."#,
    );
    insert(
        "bindings/cpp/revault_api.hpp",
        "  /** Opens an existing lockbox. */",
        "facade",
        "//",
        r#"  /** Creates and exclusively locks a native file. overwrite requires explicit replacement.
   * Example: { auto box = Lockbox::create_file(path, key, signer); box.add_file("/hello", payload); box.commit(); }
   */
  static Lockbox create_file(const std::string& path, const std::vector<std::uint8_t>& content_key, const ProfileSigningKeyPair& signer, bool overwrite = false) {
    return adopt(detail::file_handle(path, overwrite ? "replace" : "create", "content-key", content_key, nullptr, signer.native_handle(), "bytes", 64ULL << 20, "interactive", "auto", 0));
  }
  /** Opens a native file with a shared lock; supply signer for exclusive write access.
   * Close all readers before opening a writer. No Session Agent is used.
   * Example: { auto box = Lockbox::open_file(path, key); auto payload = box.get_file("/hello"); }
   */
  static Lockbox open_file(const std::string& path, const std::vector<std::uint8_t>& content_key, const ProfileSigningKeyPair* signer = nullptr) {
    return adopt(detail::file_handle(path, "open", "content-key", content_key, nullptr, signer ? signer->native_handle() : nullptr, "bytes", 64ULL << 20, "interactive", "auto", 0));
  }
"#,
    );
    let cpp = "bindings/cpp/revault_api.hpp";
    let mut source = fs::read_to_string(cpp).unwrap();
    if let Some(a) = source.find("// BEGIN generated file operation route\n") {
        let end = "// END generated file operation route\n";
        let b = a + source[a..].find(end).unwrap() + end.len();
        source.replace_range(a..b, "");
        fs::write(cpp, source).unwrap();
    }
    insert(
        cpp,
        "inline void require_compatible_abi()",
        "route",
        "//",
        r#"
inline void* file_handle(const std::string& path, const std::string& mode,
 const std::string& credential, const std::vector<std::uint8_t>& secret,
 const void* contact, const void* signer, const std::string& cache_mode,
 std::uint64_t cache_bytes, const std::string& workload, const std::string& worker, std::size_t jobs) {
 auto* handle = lockbox_file(path.data(), path.size(), mode.data(), mode.size(), credential.data(), credential.size(), secret.data(), secret.size(), contact, signer, cache_mode.data(), cache_mode.size(), cache_bytes, workload.data(), workload.size(), worker.data(), worker.size(), jobs);
 if (!handle) throw std::runtime_error(buffer_last_error());
 return handle;
}"#,
    );
    let mut body=String::from("    func lockboxFile(_ path: String, _ mode: String, _ credential: String, _ secret: Data, _ contact: UnsafeMutableRawPointer?, _ signer: UnsafeMutableRawPointer?, _ cacheMode: String, _ cacheBytes: UInt64, _ workload: String, _ worker: String, _ jobs: Int) throws -> UnsafeMutableRawPointer {\n");
    for n in strings {
        body += &format!("        return try {n}.withCString {{ {n}Pointer in\n");
    }
    body+="        return try secret.withUnsafeBytes { secretBytes in\n            guard let handle = lockbox_file(pathPointer, path.utf8.count, modePointer, mode.utf8.count, credentialPointer, credential.utf8.count, secretBytes.bindMemory(to: UInt8.self).baseAddress, secret.count, contact, signer, cacheModePointer, cacheMode.utf8.count, cacheBytes, workloadPointer, workload.utf8.count, workerPointer, worker.utf8.count, jobs) else { throw RevaultError.native(lastError()) }\n            return handle\n";
    body += &"        }\n".repeat(7);
    body += "    }";
    insert(
        "bindings/swift/Sources/RevaultAPI/RevaultAPI.swift",
        "    func lockboxOpen(",
        "route",
        "//",
        &body,
    );
}
