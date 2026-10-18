import { spawn } from 'node:child_process';
import { copyFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { buildEnvironment } from './build-environment.mjs';

const environment = buildEnvironment();
const run = (args) => new Promise((resolve, reject) => {
  const child = spawn('cargo', args, { env: environment, stdio: 'inherit' });
  child.on('error', reject);
  child.on('exit', (code) => resolve(code ?? 1));
});
const args = process.argv.slice(2);
if (process.platform === 'win32' && args[0] === 'test') {
  const buildArgs = args.includes('--') ? args.slice(0, args.indexOf('--')) : args;
  const result = await run([...buildArgs, '--no-run']);
  if (result) process.exit(result);
  const output = join('target', args.includes('--release') ? 'release' : 'debug');
  for (const library of ['onnxruntime.dll', 'onnxruntime_providers_shared.dll', 'sherpa-onnx-c-api.dll']) {
    const source = join(output, library);
    if (existsSync(source)) copyFileSync(source, join(output, 'deps', library));
  }
}
process.exitCode = await run(args);
