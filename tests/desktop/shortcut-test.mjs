import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, access, readFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';

const require = createRequire(import.meta.url);
const { chromium } = require(process.argv[2] || 'playwright');
const executable = resolve('target/debug/just-say-it.exe');
const processes = () => JSON.parse(execFileSync('powershell.exe', ['-NoProfile', '-Command',
  `ConvertTo-Json -Compress -InputObject @(Get-CimInstance Win32_Process -Filter "Name = 'just-say-it.exe'" | Select-Object ProcessId, ExecutablePath)`,
], { encoding: 'utf8', windowsHide: true }));
assert.equal(processes().length, 0, 'Quit existing app instances before this isolated test.');
const directory = await mkdtemp(resolve('artifacts/shortcuts-'));
const app = spawn(executable, ['--settings'], {
  env: { ...process.env, DICTATION_TEST_DIRECTORY: directory, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9223' },
  windowsHide: true, stdio: 'ignore',
});
const wait = (duration) => new Promise((resolve) => setTimeout(resolve, duration));
const until = async (check, label) => {
  const deadline = Date.now() + 30_000;
  while (!(await check())) {
    if (Date.now() >= deadline) {
      const observed = await status().catch(() => ({}));
      assert.fail(`${label}: ${JSON.stringify({ phase: observed.phase, message: observed.message, registered: observed.shortcuts_registered })}`);
    }
    await wait(50);
  }
};
const native = (script, args) => execFileSync('powershell.exe', ['-NoProfile', '-File', `tests/desktop/${script}.ps1`, ...args], { encoding: 'utf8', windowsHide: true }).trim();
let browser;
let page;
let fixture;
let pointer;
const rpc = (command, args) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
const status = () => rpc('get_snapshot');
const activation = () => page.getByRole('button', { name: 'Activation', exact: true }).click();
const input = () => page.getByRole('textbox', { name: 'Record / stop shortcut', exact: true });
const reset = () => page.getByRole('button', { name: 'Reset to default', exact: true });
const save = async () => {
  await page.getByRole('button', { name: 'Save settings', exact: true }).click();
  await page.getByText('Showing saved configuration and observed runtime status.', { exact: true }).waitFor();
  assert.equal(await page.getByRole('alert').count(), 0);
};
const focus = () => {
  const placed = native('focus-fixture', ['-AppProcessId', String(fixture.pid)]);
  pointer ||= placed;
};
const press = (key) => native('shortcut-key', ['-FixtureProcessId', String(fixture.pid), '-Key', key]);
const cycle = async (key) => {
  await wait(1200);
  focus();
  press(key);
  await until(async () => (await status()).phase === 'recording', `${key} did not start recording`);
  assert.equal((await status()).microphone_open, true);
  assert.equal((await status()).wake_listening, false);
  press(key);
  await until(async () => ['idle', 'pending_delivery'].includes((await status()).phase), `${key} did not finish recording`);
  assert.equal((await status()).microphone_open, false);
  await rpc('control', { action: 'discard' });
  await until(async () => (await status()).phase === 'idle', 'Recovery did not clear');
};

try {
  await until(async () => {
    try {
      browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
      page = browser.contexts().flatMap((context) => context.pages())[0];
      return !!page;
    } catch { return false; }
  }, 'Settings webview did not open');
  page.setDefaultTimeout(10_000);
  await page.waitForLoadState('networkidle');
  await until(async () => (await status()).shortcuts_registered, 'Default shortcut was not registered on first launch');
  const initial = await status();
  const defaultShortcut = initial.defaults.toggle_shortcut;
  assert.equal(defaultShortcut, 'Control+Shift+Space');
  assert.equal(initial.settings.toggle_shortcut, null);
  assert.equal('cancel_shortcut' in initial.settings, false);
  assert.equal(initial.microphone_open, false);
  await activation();
  assert.equal(await input().inputValue(), '');
  assert.equal(await input().getAttribute('placeholder'), `Default: ${defaultShortcut}`);
  assert.equal(await reset().isDisabled(), true);
  assert.equal(await page.getByRole('textbox', { name: 'Cancel recording', exact: true }).count(), 0);
  await page.screenshot({ path: join(directory, 'default.png'), fullPage: true });
  console.log('First launch: default registered, custom value unset, microphone closed, cancel field absent.');

  await rpc('save_settings', { settings: {
    ...initial.settings, microphone: 'system_default', language: 'en', wake_enabled: false,
    speech_model: resolve('artifacts/models/ggml-tiny-q5_1.bin'), no_speech_seconds: 30,
  } });
  const fixtureDirectory = await mkdtemp(join(directory, 'editor-'));
  fixture = spawn('powershell.exe', ['-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', 'tests/desktop/input-fixture.ps1', '-Directory', fixtureDirectory], { stdio: 'ignore' });
  await until(() => access(join(fixtureDirectory, 'ready')).then(() => true, () => false), 'Editor fixture did not open');
  await cycle('Space');
  console.log('Actual Control+Shift+Space started real microphone recording and the second press finished it with wake activation disabled.');

  await input().fill('Control+Shift+F9');
  await save();
  assert.equal((await status()).settings.toggle_shortcut, 'Control+Shift+F9');
  await page.reload();
  await page.waitForLoadState('networkidle');
  await activation();
  assert.equal(await input().inputValue(), 'Control+Shift+F9');
  assert.equal(await reset().isEnabled(), true);
  await cycle('F9');
  focus();
  press('Space');
  await wait(500);
  assert.equal((await status()).phase, 'idle', 'Default remained active after custom binding');
  console.log('Saved custom shortcut survived reload, started/stopped recording, and replaced the default registration.');

  await input().fill('unrecognized-shortcut');
  await page.getByRole('button', { name: 'Save settings', exact: true }).click();
  await page.getByRole('alert').waitFor();
  assert.equal((await status()).settings.toggle_shortcut, 'Control+Shift+F9');
  assert.equal((await status()).shortcuts_registered, true);
  await cycle('F9');
  console.log('Rejected invalid binding preserved the saved and actually functional custom shortcut.');

  await reset().click();
  assert.equal(await input().inputValue(), '');
  assert.equal((await status()).settings.toggle_shortcut, 'Control+Shift+F9', 'Reset applied before Save');
  await save();
  assert.equal((await status()).settings.toggle_shortcut, null);
  assert.equal((await status()).shortcuts_registered, true);
  await cycle('Space');
  focus();
  press('F9');
  await wait(500);
  assert.equal((await status()).phase, 'idle', 'Custom shortcut remained active after reset');
  await page.reload();
  await page.waitForLoadState('networkidle');
  await activation();
  assert.equal(await input().inputValue(), '');
  assert.equal(await reset().isDisabled(), true);
  const saved = JSON.parse(await readFile(join(directory, 'settings.json'), 'utf8'));
  assert.equal(saved.toggle_shortcut, null);
  assert.equal('cancel_shortcut' in saved, false);
  await page.screenshot({ path: join(directory, 'reset.png'), fullPage: true });
  console.log('Reset cleared the custom value, persisted null and restored actual default start/stop behavior.');
} finally {
  await rpc('control', { action: 'pause' }).catch(() => {});
  if (pointer && fixture?.exitCode === null) native('focus-fixture', ['-AppProcessId', String(fixture.pid), '-RestorePointer', pointer]);
  fixture?.kill();
  await rpc('control', { action: 'quit' }).catch(() => {});
  await browser?.close().catch(() => {});
  if (app.exitCode === null) app.kill();
}
console.log(`Real shortcut checks passed; isolated artifacts: ${directory}`);
