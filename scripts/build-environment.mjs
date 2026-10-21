import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const projectDirectory = fileURLToPath(new URL('../', import.meta.url));
const vulkan = JSON.parse(readFileSync(new URL('./vulkan.json', import.meta.url), 'utf8'));

export const buildEnvironment = (source = process.env) => {
  const environment = { ...source };
  if (process.platform !== 'win32') return environment;
  const localVulkan = join(projectDirectory, 'artifacts', 'build-tools', 'vulkan-sdk', vulkan.sdkVersion);
  if (!environment.VULKAN_SDK && environment['ProgramFiles(x86)'] && existsSync(join(localVulkan, 'Bin', 'glslc.exe'))) {
    environment.VULKAN_SDK = localVulkan;
  }
  const localClang = join(projectDirectory, 'artifacts', 'build-tools', 'clang', 'native');
  if (!environment.LIBCLANG_PATH && existsSync(join(localClang, 'libclang.dll'))) {
    environment.LIBCLANG_PATH = localClang;
  }
  if (environment.BINDGEN_EXTRA_CLANG_ARGS) return environment;
  const programFiles = environment['ProgramFiles(x86)'];
  if (!programFiles) throw new Error('Windows build tools location is unavailable.');
  const vswhere = join(programFiles, 'Microsoft Visual Studio', 'Installer', 'vswhere.exe');
  const installation = execFileSync(vswhere, ['-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-property', 'installationPath'], { encoding: 'utf8', windowsHide: true }).trim();
  if (!installation) throw new Error('Install the Visual Studio C++ desktop build tools.');
  const ninjaDirectory = join(installation, 'Common7', 'IDE', 'CommonExtensions', 'Microsoft', 'CMake', 'Ninja');
  if (!environment.CMAKE_GENERATOR && existsSync(join(ninjaDirectory, 'ninja.exe'))) {
    environment.CMAKE_GENERATOR = 'Ninja';
    const pathKey = Object.keys(environment).find((key) => key.toLowerCase() === 'path') || 'Path';
    environment[pathKey] = [ninjaDirectory, environment[pathKey]].filter(Boolean).join(';');
  }
  const version = readFileSync(join(installation, 'VC', 'Auxiliary', 'Build', 'Microsoft.VCToolsVersion.default.txt'), 'utf8').trim();
  const sdkRoot = join(programFiles, 'Windows Kits', '10', 'Include');
  const sdk = readdirSync(sdkRoot).filter((name) => ['ucrt', 'shared', 'um'].every((part) => existsSync(join(sdkRoot, name, part)))).sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))[0];
  if (!sdk) throw new Error('Install a Windows SDK with the C++ build tools.');
  const includes = [join(installation, 'VC', 'Tools', 'MSVC', version, 'include'), ...['ucrt', 'shared', 'um'].map((name) => join(sdkRoot, sdk, name))];
  environment.BINDGEN_EXTRA_CLANG_ARGS = includes.map((path) => `-isystem "${path}"`).join(' ');
  return environment;
};
