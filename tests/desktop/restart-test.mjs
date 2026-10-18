import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const require = createRequire(import.meta.url);
const { chromium } = require(process.argv[2] || 'playwright');
const executable = resolve('target/debug/just-say-it.exe');
const processes = () => JSON.parse(execFileSync('powershell.exe', ['-NoProfile', '-Command',
  `ConvertTo-Json -Compress -InputObject @(Get-CimInstance Win32_Process -Filter "Name = 'just-say-it.exe'" | Select-Object ProcessId, ExecutablePath)`,
], { encoding: 'utf8', windowsHide: true }));
assert.equal(processes().length, 0, 'Quit existing app instances before running this isolated test.');
const directory = await mkdtemp(resolve('artifacts/restart-'));
const environment = { ...process.env, DICTATION_TEST_DIRECTORY: directory, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9223' };
const launch = (args = []) => spawn(executable, args, { env: environment, windowsHide: true, stdio: 'ignore' });
const wait = (duration) => new Promise((resolve) => setTimeout(resolve, duration));
const until = async (check, label) => {
  const deadline = Date.now() + 30_000;
  while (!(await check())) {
    assert.ok(Date.now() < deadline, label);
    await wait(100);
  }
};
let browser;
let page;
const connect = async () => {
  await until(async () => {
    try {
      browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
      page = browser.contexts().flatMap((context) => context.pages())[0];
      return !!page;
    } catch { return false; }
  }, 'Settings webview did not open');
  page.setDefaultTimeout(10_000);
  await page.waitForLoadState('networkidle');
  await page.getByRole('heading', { name: 'Give your voice a way in' }).waitFor();
};
const rpc = (command, args) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
const status = () => rpc('get_snapshot');
const currentProcess = () => processes().find((process) => process.ExecutablePath?.toLowerCase() === executable.toLowerCase())?.ProcessId;
const native = (pid, script, args) => execFileSync('powershell.exe', ['-NoProfile', '-File', `tests/desktop/${script}.ps1`, '-AppProcessId', String(pid), ...args], { encoding: 'utf8', windowsHide: true }).trim();
const restarted = async (previous) => {
  await until(() => currentProcess() && currentProcess() !== previous && !processes().some((process) => process.ProcessId === previous), 'Restart did not replace the process');
  assert.equal(processes().length, 1, 'Restart left duplicate app processes');
  await browser.close().catch(() => {});
  await connect();
  return currentProcess();
};

launch(['--settings']);
try {
  await connect();
  let pid = currentProcess();
  assert.ok(pid);
  const initial = await status();
  const settings = { ...initial.settings, microphone: 'system_default', wake_phrase: 'hello little notebook' };
  await rpc('save_settings', { settings });
  await until(async () => (await status()).settings.wake_phrase === settings.wake_phrase, 'Settings did not apply');
  const saved = await readFile(resolve(directory, 'settings.json'), 'utf8');
  await page.getByRole('button', { name: 'Activation', exact: true }).click();
  await page.getByRole('textbox', { name: 'Wake phrase', exact: true }).fill('unsaved phrase');
  await page.getByRole('button', { name: 'Background & privacy', exact: true }).click();
  const restart = page.getByRole('button', { name: 'Restart application', exact: true });
  await restart.waitFor();
  await page.screenshot({ path: resolve(directory, 'settings-restart.png'), fullPage: true });
  await rpc('test_microphone');
  await until(async () => (await status()).microphone_open, 'Real microphone test did not open capture');
  await restart.click();
  pid = await restarted(pid);
  const fresh = await status();
  assert.deepEqual(fresh.settings, settings);
  assert.equal(fresh.microphone_open, false);
  assert.equal(fresh.whisper_loaded, false);
  assert.equal(fresh.pending_text, false);
  assert.equal(fresh.inference_runs, 0);
  assert.equal(await readFile(resolve(directory, 'settings.json'), 'utf8'), saved);
  await page.getByRole('button', { name: 'Activation', exact: true }).click();
  assert.equal(await page.getByRole('textbox', { name: 'Wake phrase', exact: true }).inputValue(), settings.wake_phrase);
  console.log('Settings button replaced the process, ended a real microphone test, cleared the draft and retained the saved settings file byte-for-byte.');

  await rpc('control', { action: 'pause' });
  await until(async () => (await status()).settings.paused, 'Pause did not apply');
  const paused = await status();
  const pausedFile = await readFile(resolve(directory, 'settings.json'), 'utf8');
  const closed = page.waitForEvent('close');
  native(pid, 'native-window', ['-Action', 'close']);
  await closed;
  console.log(native(pid, 'tray-menu', ['-Item', 'Restart application']));
  pid = await restarted(pid);
  const afterTray = await status();
  assert.deepEqual(afterTray.settings, paused.settings);
  assert.equal(afterTray.phase, 'paused');
  assert.equal(afterTray.microphone_open, false);
  assert.equal(await readFile(resolve(directory, 'settings.json'), 'utf8'), pausedFile);
  console.log('Native right-click tray menu restarted with Settings closed; saved pause and preferences survived with microphone capture closed.');

  await rpc('control', { action: 'quit' });
  await until(() => !currentProcess(), 'Quit did not end the test app');
  await browser.close();
  launch();
  await until(() => !!currentProcess(), 'Tray-only launch did not start');
  const backgroundPid = currentProcess();
  console.log(native(backgroundPid, 'tray-menu', ['-Item', 'Restart application']));
  await until(() => currentProcess() && currentProcess() !== backgroundPid && !processes().some((process) => process.ProcessId === backgroundPid), 'Tray-only restart did not replace the process');
  const backgroundWindow = JSON.parse(native(currentProcess(), 'native-window', ['-Action', 'window']));
  assert.equal(backgroundWindow.visible_settings_window, false);
  launch(['--settings']);
  await connect();
  assert.deepEqual((await status()).settings, paused.settings);
  assert.equal(processes().length, 1);
  console.log('Ordinary tray-only startup remained in the tray after restart; opening Settings reused the single fresh process.');
} finally {
  await rpc('control', { action: 'quit' }).catch(() => {});
  await browser?.close().catch(() => {});
  const pid = currentProcess();
  if (pid) native(pid, 'tray-menu', ['-Item', 'Quit']);
  await until(() => !currentProcess(), 'Test app cleanup failed');
}
console.log(`Real restart checks passed; isolated artifacts: ${directory}`);
