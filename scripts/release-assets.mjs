import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { buildEnvironment, projectDirectory } from './build-environment.mjs';

const readJson = (path) => JSON.parse(readFileSync(join(projectDirectory, path), 'utf8'));
const configuration = readJson('src-tauri/tauri.conf.json');
const frontend = readJson('package.json');
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
  cwd: projectDirectory, env: buildEnvironment(), encoding: 'utf8', windowsHide: true,
}));
const native = metadata.packages.find((entry) => entry.manifest_path.replaceAll('\\', '/').endsWith('/src-tauri/Cargo.toml'));
const tag = `v${configuration.version}`;
if (frontend.version !== configuration.version || native?.version !== configuration.version) {
  throw new Error('Frontend, native package and Tauri release versions must match.');
}
if (process.env.GITHUB_REF_TYPE === 'tag' && process.env.GITHUB_REF_NAME !== tag) {
  throw new Error(`The pushed tag must match the application version: ${tag}`);
}
const bundleDirectory = join(metadata.target_directory, 'release', 'bundle', 'nsis');
const expectedInstaller = `${configuration.productName}_${configuration.version}_x64-setup.exe`;
if (!readdirSync(bundleDirectory).includes(expectedInstaller)) throw new Error(`Installer is missing: ${expectedInstaller}`);
const output = join(projectDirectory, 'artifacts', 'release');
mkdirSync(output, { recursive: true });
const installer = join(output, expectedInstaller);
copyFileSync(join(bundleDirectory, expectedInstaller), installer);
const checksum = createHash('sha256').update(readFileSync(installer)).digest('hex');
writeFileSync(join(output, 'SHA256SUMS'), `${checksum}  ${expectedInstaller}\n`);
copyFileSync(join(projectDirectory, 'docs', 'THIRD-PARTY-NOTICES.md'), join(output, 'THIRD-PARTY-NOTICES.md'));
console.log(`Prepared ${tag}: ${expectedInstaller} and SHA256SUMS.`);
