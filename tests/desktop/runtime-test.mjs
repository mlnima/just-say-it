import { createRequire } from 'node:module';
import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, readFile, writeFile, rename, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const require = createRequire(import.meta.url);
const modulePath = process.argv[2] || 'playwright';
const { chromium } = require(modulePath);
const appProcess = Number(process.argv[3]);
const executable = resolve('target/debug/just-say-it.exe');
const wait = (duration) => new Promise((resolve) => setTimeout(resolve, duration));
const until = async (check, label, timeout = 15000) => {
  const deadline = Date.now() + timeout;
  while (!(await check())) { if (Date.now() > deadline) { const snapshot = await status().catch(() => null); throw new Error(`${label}: ${JSON.stringify({ phase: snapshot?.phase, message: snapshot?.message })}`); } await wait(50); }
};
let browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
let page = browser.contexts().flatMap((context) => context.pages())[0];
await page.waitForLoadState('networkidle');
const rpc = (command, args) => page.evaluate(async ({ command, args }) => {
  const { request } = await import('/src/bridge/commands.ts');
  return request(command, args);
}, { command, args });
const status = () => rpc('get_snapshot');
const original = await status();
if (!original.settings.paused) throw new Error('Run the fresh settings UI test first.');
await rpc('control', { action: 'resume' });
await until(async () => !(await status()).settings.paused, 'Resume failed');
const settings = {
  ...(await status()).settings, microphone: 'system_default', language: 'en',
  speech_model: resolve('artifacts/models/ggml-tiny-q5_1.bin'),
  wake_model: resolve('artifacts/models/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01'),
  wake_phrase: 'hello little notebook', wake_enabled: false, input_button: true,
  no_speech_seconds: 30, silence_seconds: 30,
  toggle_shortcut: 'Control+Shift+F9', cancel_shortcut: 'Control+Shift+F10',
};
await rpc('save_settings', { settings });
await until(async () => (await status()).phase === 'idle', 'Saved button configuration did not apply');
if ((await status()).whisper_loaded) throw new Error('Settings retained an idle Whisper model');
await page.getByRole('button', { name: 'Activation', exact: true }).click();
await page.getByRole('textbox', { name: 'Wake phrase', exact: true }).focus();
execFileSync('powershell.exe', ['-NoProfile', '-File', 'tests/desktop/native-window.ps1', '-AppProcessId', String(appProcess), '-Action', 'focus'], { encoding: 'utf8', windowsHide: true });
await until(async () => (await status()).target_status !== 'Editable input detected', 'Settings input was treated as a destination');
const ownWindow = JSON.parse(execFileSync('powershell.exe', ['-NoProfile', '-File', 'tests/desktop/native-window.ps1', '-AppProcessId', String(appProcess), '-Action', 'window'], { encoding: 'utf8', windowsHide: true }));
if (ownWindow.foreground_process_id !== appProcess || ownWindow.input_button_visible) throw new Error('Own settings input was offered as a dictation target');
const fixtureDirectory = await mkdtemp(join(tmpdir(), 'dictation-browser-'));
const fixture = spawn(process.execPath, ['tests/desktop/browser-fixture.mjs', fixtureDirectory], { env: { ...process.env, PLAYWRIGHT_MODULE_PATH: modulePath }, stdio: 'inherit', windowsHide: true });
await until(() => access(join(fixtureDirectory, 'ready')).then(() => true, () => false), 'Browser fixture not ready');
const processes = JSON.parse(await readFile(join(fixtureDirectory, 'ready'), 'utf8'));
const native = (action) => execFileSync('powershell.exe', ['-NoProfile', '-File', 'tests/desktop/native-window.ps1', '-AppProcessId', String(appProcess), '-Action', action, '-AllowedForegroundProcesses', JSON.stringify(processes)], { encoding: 'utf8', windowsHide: true }).trim();
try {
  await until(async () => (await status()).target_status === 'Editable input detected', 'Browser input was not detected');
  console.log(native('click'));
  await until(async () => (await status()).phase === 'recording', 'Input button did not start recording');
  console.log(native('click'));
  await until(async () => (await status()).phase === 'idle', 'Finish did not return to idle');
  const activatedRuns = (await status()).inference_runs;
  await rpc('control', { action: 'pause' });
  await until(async () => (await status()).phase === 'paused', 'Pause failed');
  if ((await status()).microphone_open) throw new Error('Pause retained microphone capture');
  await rpc('save_settings', { settings: { ...(await status()).settings, wake_enabled: true } });
  await rpc('control', { action: 'resume' });
  await until(async () => (await status()).wake_listening && !(await status()).whisper_loaded, 'Resume did not restore only the wake detector');
  await writeFile(join(fixtureDirectory, 'command.part'), JSON.stringify({ action: 'close' }));
  await rename(join(fixtureDirectory, 'command.part'), join(fixtureDirectory, 'command.json'));
  await until(() => access(join(fixtureDirectory, 'command.json')).then(() => false, () => true), 'Browser fixture did not close');
  const settingsClosed = page.waitForEvent('close');
  console.log(native('close'));
  await settingsClosed;
  await wait(2000);
  const before = JSON.parse(native('stats'));
  await wait(30000);
  const after = JSON.parse(native('stats'));
  const measurement = { interval_seconds: 30, cpu_percent_of_one_core: (after.cpu_seconds - before.cpu_seconds) / 30 * 100, working_set_bytes: after.working_set_bytes, private_bytes: after.private_bytes };
  await writeFile('artifacts/wake-resources.json', JSON.stringify(measurement, null, 2));
  console.log(`Tray-only wake resources: ${JSON.stringify(measurement)}`);
  spawn(executable, ['--settings'], { windowsHide: true, stdio: 'ignore' });
  await until(async () => {
    try { browser = await chromium.connectOverCDP('http://127.0.0.1:9223'); page = browser.contexts().flatMap((context) => context.pages())[0]; return !!page; } catch { return false; }
  }, 'Settings did not reopen');
  await page.waitForLoadState('networkidle');
  if ((await status()).inference_runs !== activatedRuns) throw new Error('Whisper ran during idle wake monitoring');
  if ((await status()).whisper_loaded) throw new Error('Idle wake monitoring retained Whisper');
  await rpc('control', { action: 'pause' });
  await until(async () => !(await status()).microphone_open, 'Final pause did not release the microphone');
  const paused = (await status()).settings;
  const startup = original.startup_registered;
  if (startup !== null) {
    try {
      await rpc('save_settings', { settings: { ...paused, launch_at_login: !startup } });
      if ((await status()).startup_registered !== !startup) throw new Error('Login startup registration did not change');
    } finally { await rpc('save_settings', { settings: { ...paused, launch_at_login: startup } }); }
    if ((await status()).startup_registered !== startup) throw new Error('Original login registration was not restored');
  }
  console.log('Real runtime: input offer, physical start/finish clicks, preserved focus, pause/resume, closed-settings background operation, second-instance reopening, startup registration and zero additional idle Whisper inference passed. Live microphone audio is uncontrolled; synthetic silence is covered separately.');
} finally {
  fixture.kill();
  await rpc('control', { action: 'pause' }).catch(() => {});
  await browser.close();
}
