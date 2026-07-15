import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);

function nativeTarget() {
  const key = `${process.platform}-${process.arch}`;
  const targets = {
    'linux-x64': ['linux-x86_64-gnu', 'librevault_api.so'],
    'linux-arm64': ['linux-aarch64-gnu', 'librevault_api.so'],
    'darwin-x64': ['macos-x86_64', 'librevault_api.dylib'],
    'darwin-arm64': ['macos-aarch64', 'librevault_api.dylib'],
    'win32-x64': ['windows-x86_64-msvc', 'revault_api.dll'],
    'win32-arm64': ['windows-aarch64-msvc', 'revault_api.dll'],
  };
  const target = targets[key];
  if (target == null) throw new Error(`reVault does not publish a native library for ${key}`);
  if (process.platform === 'linux' && process.report?.getReport()?.header?.glibcVersionRuntime == null) {
    throw new Error('reVault Linux packages currently require glibc');
  }
  return target;
}

export function nativeLibraryPath() {
  if (process.env.REVAULT_LIBRARY) return process.env.REVAULT_LIBRARY;
  const [target, library] = nativeTarget();
  const packageName = `@onepub/revault-api-native-${target}`;
  try {
    const manifest = require.resolve(`${packageName}/package.json`);
    const candidate = path.join(path.dirname(manifest), 'lib', library);
    if (fs.existsSync(candidate)) return candidate;
  } catch (error) {
    if (error?.code !== 'MODULE_NOT_FOUND') throw error;
  }
  const bundled = path.join(path.dirname(fileURLToPath(import.meta.url)), 'native', target, library);
  if (fs.existsSync(bundled)) return bundled;
  throw new Error(`revault-api native carrier ${packageName} is missing; set REVAULT_LIBRARY for development`);
}
