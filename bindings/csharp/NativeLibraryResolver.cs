using System.Reflection;
using System.Runtime.InteropServices;

namespace Revault;

internal static class NativeLibraryResolver
{
    private static readonly object Gate = new();
    private static string? selectedPath;
    private static IntPtr selectedHandle;
    private static bool nativeUsed;

    internal static void Configure(string? explicitPath)
    {
        if (explicitPath is not null && explicitPath.Length == 0)
            throw new ArgumentException("Native library path must not be empty.", nameof(explicitPath));
        var inherited = Environment.GetEnvironmentVariable("REVAULT_LIBRARY");
        var path = explicitPath ?? (string.IsNullOrEmpty(inherited) ? null : inherited);
        if (path is null) return;

        lock (Gate)
        {
            if (explicitPath is null && selectedPath is not null) return;
            if (selectedPath == path) return;
            if (nativeUsed || selectedPath is not null)
                throw new InvalidOperationException("The process-wide reVault native library is already selected.");
            selectedHandle = System.Runtime.InteropServices.NativeLibrary.Load(path);
            selectedPath = path;
            System.Runtime.InteropServices.NativeLibrary.SetDllImportResolver(
                typeof(RevaultNative).Assembly,
                Resolve);
        }
    }

    internal static void MarkUsed()
    {
        lock (Gate) nativeUsed = true;
    }

    private static IntPtr Resolve(string libraryName, Assembly assembly, DllImportSearchPath? searchPath) =>
        libraryName == "revault_api" ? selectedHandle : IntPtr.Zero;
}
