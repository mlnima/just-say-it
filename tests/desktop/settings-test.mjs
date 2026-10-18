import { createRequire } from 'node:module';
import { mkdir } from 'node:fs/promises';

const require = createRequire(import.meta.url);
const { chromium } = require(process.argv[2] || 'playwright');
const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts().flatMap((context) => context.pages())[0];
if (!page) throw new Error('Launch the isolated desktop app with a settings window first.');
if (!page.url().includes('127.0.0.1:1420')) await page.goto('http://127.0.0.1:1420');
const errors = [];
page.on('pageerror', (error) => errors.push(error.message));
await page.waitForLoadState('networkidle');
await page.getByRole('heading', { name: 'Give your voice a way in' }).waitFor();
for (const select of await page.locator('select').all()) {
  if (await select.inputValue() !== '') throw new Error('An unset dropdown is falsely selected.');
}
await page.getByRole('button', { name: 'Input & recovery', exact: true }).click();
const inputButton = page.getByRole('switch', { name: 'Show the input button' });
if (await inputButton.getAttribute('aria-checked') !== 'false') throw new Error('Unset input button falsely enabled.');
await inputButton.click();
await page.getByRole('button', { name: 'Save settings', exact: true }).click();
await page.getByText('Settings saved and applied.', { exact: true }).waitFor();
await page.reload();
await page.getByRole('button', { name: 'Input & recovery', exact: true }).click();
if (await page.getByRole('switch', { name: 'Show the input button' }).getAttribute('aria-checked') !== 'true') throw new Error('Saved setting did not survive webview reload.');
await page.getByRole('button', { name: 'Activation', exact: true }).click();
await page.getByRole('textbox', { name: 'Wake phrase', exact: true }).fill('hello little notebook');
await page.getByRole('button', { name: 'Save settings', exact: true }).click();
await page.getByText('Settings saved and applied.', { exact: true }).waitFor();
await page.getByRole('button', { name: 'Pause', exact: true }).click();
await page.getByRole('button', { name: 'Resume', exact: true }).waitFor();
await page.getByText('Microphone closed', { exact: true }).waitFor();
await mkdir('artifacts/screenshots', { recursive: true });
await page.screenshot({ path: 'artifacts/screenshots/activation.png', fullPage: true });
await page.getByRole('button', { name: 'Audio & model', exact: true }).click();
await page.screenshot({ path: 'artifacts/screenshots/settings.png', fullPage: true });
if (errors.length) throw new Error(errors.join('; '));
console.log('Real desktop UI: unset selections, saving, reload persistence, editable phrase, pause and closed microphone passed.');
await browser.close();
