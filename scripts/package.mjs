import { execFileSync, spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { parseArgs } from 'node:util';
import { buildEnvironment, projectDirectory } from './build-environment.mjs';

const environment = buildEnvironment();
const run = (args) => new Promise((resolve, reject) => {
  const child = spawn(process.execPath, ['node_modules/@tauri-apps/cli/tauri.js', ...args], { cwd: projectDirectory, env: environment, stdio: 'inherit', windowsHide: true });
  child.on('error', reject);
  child.on('exit', (code) => resolve(code ?? 1));
});
const args = process.argv.slice(2);
const result = await run(['build', '--no-bundle', ...args.filter((arg) => arg !== '--no-bundle')]);
if (result || args.includes('--no-bundle')) process.exit(result);
const bundleArgs = args.includes('--') ? args.slice(0, args.indexOf('--')) : args;
const { values } = parseArgs({ args: bundleArgs, strict: false, allowPositionals: true, options: {
  target: { type: 'string', short: 't' }, debug: { type: 'boolean', short: 'd' },
} });
const bundle = ['bundle', ...bundleArgs];
if (process.platform === 'win32') {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], { cwd: projectDirectory, env: environment, encoding: 'utf8', windowsHide: true }));
  const target = values.target || environment.CARGO_BUILD_TARGET;
  const output = join(metadata.target_directory, ...(target ? [target] : []), values.debug ? 'debug' : 'release');
  const libraries = ['onnxruntime.dll', 'onnxruntime_providers_shared.dll', 'sherpa-onnx-c-api.dll'];
  const resources = Object.fromEntries(libraries.map((name) => {
    const path = join(output, name);
    if (!existsSync(path)) throw new Error(`Native runtime library is missing: ${path}`);
    return [path, name];
  }));
  resources[join(projectDirectory, 'docs', 'THIRD-PARTY-NOTICES.md')] = 'THIRD-PARTY-NOTICES.md';
  const hasBundles = bundleArgs.some((arg) => arg === '-b' || arg === '--bundles' || arg.startsWith('--bundles='));
  bundle.push(...(hasBundles ? [] : ['--bundles', 'nsis']), '--config', JSON.stringify({ bundle: { resources } }));
}
process.exitCode = await run(bundle);
