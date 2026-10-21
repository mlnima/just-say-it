import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, readFile, writeFile, rename, access, unlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const require = createRequire(import.meta.url);
const { chromium } = require(process.argv[2] || 'playwright');
const appProcess = Number(process.argv[3]);
assert(appProcess > 0, 'Pass the isolated running app process ID.');
const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts().flatMap((context) => context.pages())[0];
await page.waitForLoadState('networkidle');
const rpc = (command, args) => page.evaluate(({ command, args }) =>
  window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
const status = () => rpc('get_snapshot');
const original = (await status()).settings;
const wait = (duration) => new Promise((resolve) => setTimeout(resolve, duration));
const until = async (check, label) => {
  const deadline = Date.now() + 15000;
  while (!(await check())) {
    if (Date.now() >= deadline) {
      const observed = await status();
      assert.fail(`${label}: ${JSON.stringify({ phase: observed.phase, target_status: observed.target_status, ...native() })}`);
    }
    await wait(50);
  }
};
const exists = (path) => access(path).then(() => true, () => false);
const native = () => JSON.parse(execFileSync('powershell.exe', [
  '-NoProfile', '-File', 'tests/desktop/native-window.ps1',
  '-AppProcessId', String(appProcess), '-Action', 'window',
], { encoding: 'utf8', windowsHide: true }));
const action = async (directory, command) => {
  await writeFile(join(directory, 'command.part'), JSON.stringify(command));
  await rename(join(directory, 'command.part'), join(directory, 'command.json'));
  await until(async () => !(await exists(join(directory, 'command.json'))), 'Fixture command timed out');
};

try {
  await rpc('save_settings', { settings: { ...original, input_button: true, paused: false, wake_enabled: false } });
  for (const [script, readOnlyIndex] of [['input-fixture.ps1', 3], ['document-fixture.ps1', 2]]) {
    const directory = await mkdtemp(join(tmpdir(), 'dictation-detection-'));
    const fixture = spawn('powershell.exe', ['-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', `tests/desktop/${script}`, '-Directory', directory], {
      stdio: 'inherit',
    });
    let pointer;
    try {
      await until(() => exists(join(directory, 'ready')), 'Fixture did not open');
      pointer = execFileSync('powershell.exe', [
        '-NoProfile', '-File', 'tests/desktop/focus-fixture.ps1', '-AppProcessId', String(fixture.pid),
      ], { encoding: 'utf8', windowsHide: true }).trim();
      await until(async () => (await status()).target_status === 'Editable input detected', 'Editable field was not detected');
      let observed = native();
      assert.equal(observed.foreground_process_id, fixture.pid, 'An unrelated application is focused');
      assert.equal(observed.input_button_visible, true, 'Dictation button is missing');
      await action(directory, { action: 'focus', index: 1 });
      await until(async () => (await status()).target_status === 'Editable input detected', 'Second field was not detected');
      observed = native();
      assert.equal(observed.foreground_process_id, fixture.pid);
      assert.equal(observed.input_button_visible, true);
      await action(directory, { action: 'focus', index: readOnlyIndex });
      await until(async () => (await status()).target_status === 'No supported editable input detected', 'Read-only field was accepted');
      assert.equal(native().input_button_visible, false, 'Read-only field still offers dictation');
      if (script === 'input-fixture.ps1') {
        await action(directory, { action: 'focus', index: 2 });
        await until(async () => (await status()).target_status === 'Protected or uninspectable field', 'Password field was accepted');
        assert.equal(native().input_button_visible, false, 'Password field still offers dictation');
      }
      await action(directory, { action: 'focus', index: 0 });
      await until(async () => (await status()).target_status === 'Editable input detected', 'Button did not return on an editable field');
      assert.equal(native().input_button_visible, true);
      await action(directory, { action: 'read' });
      const result = JSON.parse((await readFile(join(directory, 'result.json'), 'utf8')).replace(/^\uFEFF/, ''));
      assert.deepEqual(result, { first: '', second: '' }, 'Detection changed fixture text');
      await unlink(join(directory, 'result.json'));
      console.log(`${script}: real input button appears, follows focus, hides on protected/read-only fields, and returns without modifying text.`);
    } finally {
      if (pointer) execFileSync('powershell.exe', [
        '-NoProfile', '-File', 'tests/desktop/focus-fixture.ps1', '-AppProcessId', String(fixture.pid), '-RestorePointer', pointer,
      ], { encoding: 'utf8', windowsHide: true });
      fixture.kill();
    }
  }
  assert.equal((await status()).microphone_open, false, 'Detection opened the microphone');
  assert.equal((await status()).input_detection_supported, true, 'Input event registration failed');
} finally {
  await rpc('save_settings', { settings: original }).catch(() => {});
  await browser.close();
}
