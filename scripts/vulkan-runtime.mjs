import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const project = fileURLToPath(new URL('../', import.meta.url));
const config = JSON.parse(readFileSync(new URL('./vulkan.json', import.meta.url), 'utf8'));
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');

export const prepareVulkanRuntime = async () => {
  if (process.platform !== 'win32') return;
  const directory = join(project, 'artifacts', 'build-tools', 'vulkan-runtime');
  const archive = join(directory, 'runtime.zip');
  mkdirSync(directory, { recursive: true });
  if (!existsSync(archive) || digest(readFileSync(archive)) !== config.runtimeSha256) {
    const response = await fetch(config.runtimeUrl, { signal: AbortSignal.timeout(120_000) });
    if (!response.ok) throw new Error(`Vulkan runtime download failed: ${response.status}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    if (digest(bytes) !== config.runtimeSha256) throw new Error('Vulkan runtime integrity check failed.');
    const temporary = `${archive}.${process.pid}.tmp`;
    writeFileSync(temporary, bytes);
    renameSync(temporary, archive);
  }
  execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-File', join(project, 'scripts', 'extract-vulkan-runtime.ps1'), '-Archive', archive, '-Destination', directory, '-ArchiveDirectory', config.runtimeArchiveDirectory], { stdio: 'inherit', windowsHide: true });
};
