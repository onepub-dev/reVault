$ErrorActionPreference = 'Stop'

$commit = '1ee778a4e37122d8ca7d5733c590a47dafd6b15c'
$expected = '3245c6d185612155835b13df4aca5a1a1b22c2cc91bf8141233fb379f5bfd78a'
$archive = Join-Path $env:RUNNER_TEMP "luajit-$commit.zip"
$sourceRoot = Join-Path $env:RUNNER_TEMP 'luajit-source'
$installRoot = Join-Path $env:RUNNER_TEMP 'luajit-arm64'
$bin = Join-Path $installRoot 'bin'
$hostArch = $env:VSCMD_ARG_HOST_ARCH.ToLowerInvariant()
$targetArch = $env:VSCMD_ARG_TGT_ARCH.ToLowerInvariant()
$linker = Join-Path $env:VCToolsInstallDir "bin\Host$hostArch\$targetArch\link.exe"
if (-not (Test-Path $linker)) {
  throw "MSVC linker is missing: $linker"
}
$env:PATH = "$(Split-Path $linker);$env:PATH"

Invoke-WebRequest `
  -Uri "https://github.com/LuaJIT/LuaJIT/archive/$commit.zip" `
  -OutFile $archive
$actual = (Get-FileHash -Algorithm SHA256 $archive).Hash.ToLowerInvariant()
if ($actual -ne $expected) {
  throw "LuaJIT source archive checksum mismatch: $actual"
}

Expand-Archive -Path $archive -DestinationPath $sourceRoot
$source = Join-Path $sourceRoot "LuaJIT-$commit"
Push-Location (Join-Path $source 'src')
try {
  & cmd.exe /d /c msvcbuild.bat
  if ($LASTEXITCODE -ne 0 -or
      -not (Test-Path (Join-Path $source 'src/luajit.exe')) -or
      -not (Test-Path (Join-Path $source 'src/lua51.dll'))) {
    throw "LuaJIT ARM64 build failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}

New-Item -ItemType Directory -Force -Path $bin | Out-Null
Copy-Item (Join-Path $source 'src/luajit.exe') (Join-Path $bin 'lua.exe')
Copy-Item (Join-Path $source 'src/lua51.dll') $bin
$bin | Out-File -FilePath $env:GITHUB_PATH -Encoding utf8 -Append
