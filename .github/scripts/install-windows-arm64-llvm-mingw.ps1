$ErrorActionPreference = 'Stop'

$version = '20260616'
$archiveName = "llvm-mingw-$version-ucrt-aarch64.zip"
$expectedSha256 = '312593669435bd0bfc1a43ac3fba23c8b27e0610bade88b2738e5a01702a99ba'
$root = Join-Path $env:RUNNER_TOOL_CACHE "llvm-mingw-$version-ucrt-aarch64"
$compiler = Join-Path $root 'bin\aarch64-w64-mingw32-clang.exe'

if (-not (Test-Path $compiler)) {
  $archive = Join-Path $env:RUNNER_TEMP $archiveName
  Invoke-WebRequest `
    -Uri "https://github.com/mstorsjo/llvm-mingw/releases/download/$version/$archiveName" `
    -OutFile $archive
  $actualSha256 = (Get-FileHash -Algorithm SHA256 $archive).Hash.ToLowerInvariant()
  if ($actualSha256 -ne $expectedSha256) {
    throw "LLVM-MinGW archive checksum mismatch: $actualSha256"
  }
  Expand-Archive -Path $archive -DestinationPath $env:RUNNER_TOOL_CACHE -Force
}

if (-not (Test-Path $compiler)) {
  throw "LLVM-MinGW ARM64 compiler is missing after installation: $compiler"
}

"REVAULT_LLVM_MINGW_ROOT=$root" |
  Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append
