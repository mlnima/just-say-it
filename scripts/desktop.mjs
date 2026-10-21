import { spawn } from 'node:child_process';
import { buildEnvironment } from './build-environment.mjs';
import { prepareVulkanRuntime } from './vulkan-runtime.mjs';

await prepareVulkanRuntime();
const child = spawn(process.execPath, ['node_modules/@tauri-apps/cli/tauri.js', ...process.argv.slice(2)], { env: buildEnvironment(), stdio: 'inherit' });
child.on('error', (error) => { console.error(error.message); process.exitCode = 1; });
child.on('exit', (code) => { process.exitCode = code ?? 1; });
