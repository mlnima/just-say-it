import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir } from 'node:fs/promises';

const require = createRequire(import.meta.url);
const { chromium } = require(process.argv[2] || 'playwright');
const directory = 'artifacts/theme-change';
await mkdir(directory, { recursive: true });
const desktop = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = desktop.contexts().flatMap((context) => context.pages())[0];
if (!page) throw new Error('Launch the isolated development app with a settings window first.');
page.setDefaultTimeout(10_000);
const errors = [];
page.on('pageerror', (error) => errors.push(error.message));
const expectedBackground = { dark: 'rgb(20, 27, 24)', light: 'rgb(246, 247, 242)' };
const assertTheme = async (target, theme) => {
  await target.waitForFunction((theme) => getComputedStyle(document.documentElement).colorScheme === theme, theme);
  assert.equal(await target.locator('html').evaluate((element) => getComputedStyle(element).backgroundColor), expectedBackground[theme]);
};

try {
  await page.getByRole('heading', { name: 'Give your voice a way in' }).waitFor();
  const observed = await page.evaluate(async () => ({
    theme: await window.__TAURI_INTERNALS__.invoke('plugin:window|theme', { label: 'settings' }),
    settings: (await window.__TAURI_INTERNALS__.invoke('get_snapshot')).settings,
  }));
  await assertTheme(page, observed.theme || 'dark');
  console.log(`Real Windows settings webview follows the native theme: ${observed.theme}.`);

  // Inspect each palette in the real UI without changing the OS or saved preferences.
  for (const theme of ['dark', 'light']) {
    await page.evaluate((theme) => { document.documentElement.dataset.theme = theme; }, theme);
    await assertTheme(page, theme);
    for (const section of ['Audio & model', 'Activation', 'Input & recovery', 'Background & privacy']) {
      await page.getByRole('button', { name: section, exact: true }).click();
      await assertTheme(page, theme);
      const sectionId = section.split(' ')[0].toLowerCase();
      await page.screenshot({ path: `${directory}/${theme}-${sectionId}.png`, fullPage: true });
    }
    await page.getByRole('button', { name: 'Audio & model', exact: true }).click();
    const select = page.getByRole('combobox', { name: 'Microphone', exact: true });
    assert.equal(await select.inputValue(), observed.settings.microphone || '');
    const scheme = await select.evaluate((element) => getComputedStyle(element).colorScheme);
    assert.equal(scheme, theme);
  }
  assert.deepEqual(await page.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('get_snapshot')).settings), observed.settings);
  assert.deepEqual(errors, []);
  console.log('Both palettes across all four real settings sections, native form colors and unchanged preferences passed.');
} finally {
  await page.emulateMedia({ colorScheme: null });
  await page.reload();
  await desktop.close();
}

// Exercise native theme events, stale lookups and hook cleanup with Tauri's IPC test adapter.
const browser = await chromium.launch({ channel: 'msedge', headless: true });
try {
  const probe = await browser.newPage();
  probe.setDefaultTimeout(10_000);
  probe.on('pageerror', (error) => console.error(`Theme hook fixture: ${error.message}`));
  probe.on('console', (message) => { if (message.type() === 'error') console.error(message.text()); });
  const origin = 'http://127.0.0.1:1420';
  await probe.route(`${origin}/theme-hook-test`, (route) => route.fulfill({
    contentType: 'text/html',
    body: `<!doctype html><html><head><link rel="stylesheet" href="/src/styles/theme.css"><link rel="stylesheet" href="/src/styles/base.css"></head><body><div id="root"></div>
<script type="module">
import React from '/node_modules/.vite/deps/react.js';
import ReactDOM from '/node_modules/.vite/deps/react-dom_client.js';
import { mockIPC, mockWindows } from '/node_modules/@tauri-apps/api/mocks.js';
import { emit } from '/node_modules/@tauri-apps/api/event.js';
window.isTauri = true;
window.themeProbe = { theme: 'dark', fail: false, pending: false, resolve: null };
mockIPC((command) => {
  if (command !== 'plugin:window|theme') throw new Error(command);
  if (window.themeProbe.fail) throw new Error('Theme lookup unavailable');
  return window.themeProbe.pending ? new Promise((resolve) => { window.themeProbe.resolve = resolve; }) : window.themeProbe.theme;
}, { shouldMockEvents: true });
mockWindows('settings');
const { useSystemTheme } = await import('/src/bridge/useSystemTheme.ts');
const Probe = () => { useSystemTheme(); return null; };
window.themeProbe.root = ReactDOM.createRoot(document.getElementById('root'));
window.themeProbe.root.render(React.createElement(React.StrictMode, null, React.createElement(Probe)));
window.themeProbe.emit = (theme) => emit('tauri://theme-changed', theme);
</script></body></html>`,
  }));
  await probe.goto(`${origin}/theme-hook-test`);
  await probe.waitForFunction(() => document.documentElement.dataset.theme === 'dark');
  for (const theme of ['light', 'dark', 'light']) {
    await probe.evaluate((theme) => window.themeProbe.emit(theme), theme);
    await assertTheme(probe, theme);
  }
  await probe.emulateMedia({ colorScheme: 'light' });
  for (const fail of [false, true]) {
    await probe.evaluate(async (fail) => {
      await window.themeProbe.emit('light');
      window.themeProbe.theme = null;
      window.themeProbe.fail = fail;
      window.dispatchEvent(new Event('focus'));
    }, fail);
    await assertTheme(probe, 'dark');
  }
  await probe.evaluate(() => { window.themeProbe.fail = false; });
  await probe.evaluate(() => { window.themeProbe.pending = true; window.dispatchEvent(new Event('focus')); });
  await probe.waitForFunction(() => !!window.themeProbe.resolve);
  await probe.evaluate(async () => { await window.themeProbe.emit('dark'); window.themeProbe.resolve('light'); });
  await assertTheme(probe, 'dark');
  await probe.evaluate(() => window.themeProbe.root.unmount());
  await probe.waitForFunction(() => window.__TAURI_INTERNALS__.callbacks.size === 0);
  await probe.evaluate(() => window.themeProbe.emit('light'));
  await assertTheme(probe, 'dark');
  console.log('Theme event updates, null/error dark fallback, stale-query protection and StrictMode listener cleanup passed with the IPC test adapter.');
} finally {
  await browser.close();
}
